//! Torn treasure maps, and the routes they sketch along the old track and off it: "past the split
//! oak, left at the mossy stone, over the stream, twelve paces from the old milestone".
//!
//! A map is only its seed: the same map always sketches the same route, and every map a
//! different one. A route is a few forks and then a dig. At each fork the ways part, two or three
//! of them, with landmarks beside them, far down them, or standing in the fork itself, and a
//! stream across one of them; the map says which way in a line naming what to look for, and the
//! person matches the line to what is there. Some landmarks have look-alikes standing nearby (an
//! oak by the split oak, a grey stone by the mossy one), and now and then a word on the map has
//! faded. Every line, read in full, names one way and only one; every other way leads to a heap
//! in a nook, so a wrong turn is never wasted. At the end, the old milestone, and four places
//! that might be dug: the map says how many paces from the milestone, and by what.

use crate::dice::Dice;

/// Something a map can name to look out for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Landmark {
    SplitOak,
    Oak,
    MossyStone,
    GreyStone,
    HollowStump,
    Stump,
    Holly,
    Birch,
    Woodpile,
    Gatepost,
}

impl Landmark {
    pub const ALL: [Self; 10] = [
        Self::SplitOak,
        Self::Oak,
        Self::MossyStone,
        Self::GreyStone,
        Self::HollowStump,
        Self::Stump,
        Self::Holly,
        Self::Birch,
        Self::Woodpile,
        Self::Gatepost,
    ];

    /// What the map calls it.
    pub fn name(self) -> &'static str {
        match self {
            Self::SplitOak => "the split oak",
            Self::Oak => "the oak",
            Self::MossyStone => "the mossy stone",
            Self::GreyStone => "the grey stone",
            Self::HollowStump => "the hollow stump",
            Self::Stump => "the stump",
            Self::Holly => "the holly",
            Self::Birch => "the birch",
            Self::Woodpile => "the woodpile",
            Self::Gatepost => "the old gatepost",
        }
    }

    /// What it could be mistaken for.
    pub fn lookalike(self) -> Option<Self> {
        match self {
            Self::SplitOak => Some(Self::Oak),
            Self::Oak => Some(Self::SplitOak),
            Self::MossyStone => Some(Self::GreyStone),
            Self::GreyStone => Some(Self::MossyStone),
            Self::HollowStump => Some(Self::Stump),
            Self::Stump => Some(Self::HollowStump),
            _ => None,
        }
    }
}

/// Which way a way goes, from the fork.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dir {
    Left,
    Ahead,
    Right,
}

impl Dir {
    pub fn name(self) -> &'static str {
        match self {
            Self::Left => "the left-hand way",
            Self::Ahead => "the way straight on",
            Self::Right => "the right-hand way",
        }
    }
}

/// Where a landmark stands at a fork.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Spot {
    /// Beside the ways: between way `k - 1` and way `k`, so `0` is left of the first way and the
    /// number of ways is right of the last.
    Beside(usize),
    /// Far off down a way, small in the distance.
    Far(usize),
    /// In the fork itself, where the ways part.
    Fork,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Placed {
    pub landmark: Landmark,
    pub spot: Spot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Right,
}

/// A line of a map: which way to take at a fork.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Clue {
    /// The way that goes past it.
    Past(Landmark),
    /// It stands in the fork: the leftmost way or the rightmost.
    Turn(Side, Landmark),
    /// The way the stream crosses.
    OverStream,
    /// The way with the first on its left and the second on its right.
    Between(Landmark, Landmark),
}

/// One fork on a route.
#[derive(Clone, Debug, PartialEq)]
pub struct Fork {
    /// The ways, left to right.
    pub ways: Vec<Dir>,
    /// The one to take.
    pub right: usize,
    /// The way the stream crosses, if one does.
    pub stream: Option<usize>,
    pub landmarks: Vec<Placed>,
    pub clue: Clue,
    /// Which word of the line has faded, if one has: the first or the second it names.
    pub faded: Option<usize>,
}

/// Something by a place that might be dug.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Feature {
    TwistedRoot,
    FlatStone,
    Foxgloves,
    Molehill,
    Ferns,
}

impl Feature {
    pub const ALL: [Self; 5] = [
        Self::TwistedRoot,
        Self::FlatStone,
        Self::Foxgloves,
        Self::Molehill,
        Self::Ferns,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::TwistedRoot => "the twisted root",
            Self::FlatStone => "the flat stone",
            Self::Foxgloves => "the foxgloves",
            Self::Molehill => "the molehill",
            Self::Ferns => "the ferns",
        }
    }
}

/// A place that might be dug, near the old milestone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DigSpot {
    pub paces: u32,
    pub feature: Feature,
    /// Where it is in the scene.
    pub at: (f32, f32),
}

/// The end of a route: where to dig.
#[derive(Clone, Debug, PartialEq)]
pub struct Dig {
    pub spots: Vec<DigSpot>,
    pub right: usize,
    /// Which word has faded, if one has: the paces, or what it is by.
    pub faded: Option<usize>,
}

/// Where the old milestone stands at the dig, and how far a pace is.
pub const MILESTONE: (f32, f32) = (112.0, 160.0);
const PACE: f32 = 5.6;
/// The directions dig spots lie in from the milestone, as angles across the scene.
const BEARINGS: [f32; 4] = [-0.2, 0.32, 0.9, 1.5];
const PACES: [u32; 5] = [4, 7, 9, 12, 15];

/// A whole route.
#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    pub forks: Vec<Fork>,
    pub dig: Dig,
}

impl Fork {
    /// Every way a line names, read in full.
    pub fn matching(&self, clue: Clue) -> Vec<usize> {
        let ways = 0..self.ways.len();
        let at = |landmark: Landmark| {
            self.landmarks
                .iter()
                .filter(move |placed| placed.landmark == landmark)
                .map(|placed| placed.spot)
        };
        match clue {
            Clue::Past(landmark) => ways
                .filter(|&way| {
                    at(landmark).any(|spot| match spot {
                        Spot::Far(far) => far == way,
                        Spot::Beside(beside) => beside == way || beside == way + 1,
                        Spot::Fork => false,
                    })
                })
                .collect(),
            Clue::Turn(side, landmark) => {
                if !at(landmark).any(|spot| spot == Spot::Fork) {
                    return Vec::new();
                }
                match side {
                    Side::Left => vec![0],
                    Side::Right => vec![self.ways.len() - 1],
                }
            }
            Clue::OverStream => self.stream.into_iter().collect(),
            Clue::Between(left, right) => ways
                .filter(|&way| {
                    at(left).any(|spot| spot == Spot::Beside(way))
                        && at(right).any(|spot| spot == Spot::Beside(way + 1))
                })
                .collect(),
        }
    }

    /// Every way the line could name, as far as it can be read: with its faded word unread, any
    /// way it might be.
    pub fn might_be(&self, read_faded: bool) -> Vec<usize> {
        let faded = self.faded.filter(|_| !read_faded);
        let mut ways: Vec<usize> = match (self.clue, faded) {
            (clue, None) => self.matching(clue),
            (Clue::Past(_), Some(_)) => (0..self.ways.len()).collect(),
            (Clue::Turn(_, landmark), Some(0)) => [Side::Left, Side::Right]
                .into_iter()
                .flat_map(|side| self.matching(Clue::Turn(side, landmark)))
                .collect(),
            (Clue::Turn(side, _), Some(_)) => match side {
                Side::Left => vec![0],
                Side::Right => vec![self.ways.len() - 1],
            },
            (Clue::OverStream, Some(_)) => self.stream.into_iter().collect(),
            (Clue::Between(left, right), Some(which)) => Landmark::ALL
                .into_iter()
                .flat_map(|other| {
                    let clue = if which == 0 {
                        Clue::Between(other, right)
                    } else {
                        Clue::Between(left, other)
                    };
                    self.matching(clue)
                })
                .collect(),
        };
        ways.sort_unstable();
        ways.dedup();
        ways
    }

    /// The line on the map, with its faded word shown as faded unless it can be read.
    pub fn line(&self, read_faded: bool) -> String {
        let faded = |slot: usize| self.faded == Some(slot) && !read_faded;
        let word = |slot: usize, word: &str| {
            if faded(slot) {
                "\u{2026}".to_owned()
            } else {
                word.to_owned()
            }
        };
        match self.clue {
            Clue::Past(landmark) => format!("Past {}.", word(0, landmark.name())),
            Clue::Turn(side, landmark) => {
                let side = match side {
                    Side::Left => "Left",
                    Side::Right => "Right",
                };
                format!("{} at {}.", word(0, side), word(1, landmark.name()))
            }
            Clue::OverStream => "Over the stream.".to_owned(),
            Clue::Between(left, right) => format!(
                "Between {} and {}.",
                word(0, left.name()),
                word(1, right.name())
            ),
        }
    }

    /// The landmark the line is about, if it names one, and whether it stands far off.
    pub fn sought(&self) -> Option<(Landmark, Spot)> {
        let landmark = match self.clue {
            Clue::Past(landmark) | Clue::Turn(_, landmark) | Clue::Between(landmark, _) => landmark,
            Clue::OverStream => return None,
        };
        self.landmarks
            .iter()
            .find(|placed| placed.landmark == landmark)
            .map(|placed| (landmark, placed.spot))
    }
}

impl Dig {
    /// The line on the map, with its faded word shown as faded unless it can be read.
    pub fn line(&self, read_faded: bool) -> String {
        let spot = &self.spots[self.right];
        let paces = if self.faded == Some(0) && !read_faded {
            "\u{2026}".to_owned()
        } else {
            let number = number(spot.paces);
            let mut letters = number.chars();
            letters
                .next()
                .map(|first| first.to_uppercase().chain(letters).collect::<String>())
                .unwrap_or_default()
        };
        let by = if self.faded == Some(1) && !read_faded {
            "\u{2026}"
        } else {
            spot.feature.name()
        };
        format!("{paces} paces from the old milestone, by {by}. Dig!")
    }

    /// Every place the line could mean, as far as it can be read.
    pub fn might_be(&self, read_faded: bool) -> Vec<usize> {
        let right = &self.spots[self.right];
        let faded = self.faded.filter(|_| !read_faded);
        (0..self.spots.len())
            .filter(|&index| {
                let spot = &self.spots[index];
                let paces = faded == Some(0) || spot.paces == right.paces;
                let by = faded == Some(1) || spot.feature == right.feature;
                paces && by
            })
            .collect()
    }
}

/// A number of paces in words.
pub fn number(paces: u32) -> &'static str {
    match paces {
        4 => "four",
        7 => "seven",
        9 => "nine",
        12 => "twelve",
        15 => "fifteen",
        _ => "a few",
    }
}

impl Route {
    /// The route a map with this seed sketches: always the same one for the same map.
    pub fn of(map: u64) -> Self {
        let mut dice = Dice::new(map ^ 0x7ea5_u64.rotate_left(40));
        let legs = 3 + usize::from(dice.chance(0.4));
        let mut forks: Vec<Fork> = Vec::with_capacity(legs);
        let mut last: Option<u8> = None;
        for leg in 0..legs {
            let fork = fork(leg, last, &mut dice);
            last = Some(kind(fork.clue));
            forks.push(fork);
        }
        let dig = dig(&mut dice);
        Self { forks, dig }
    }

    /// Every line of the map, fork by fork and then the dig.
    pub fn lines(&self, read_faded: bool) -> Vec<String> {
        self.forks
            .iter()
            .map(|fork| fork.line(read_faded))
            .chain(std::iter::once(self.dig.line(read_faded)))
            .collect()
    }
}

fn kind(clue: Clue) -> u8 {
    match clue {
        Clue::Past(_) => 0,
        Clue::Turn(..) => 1,
        Clue::OverStream => 2,
        Clue::Between(..) => 3,
    }
}

fn choose<T: Copy>(from: &[T], dice: &mut Dice) -> T {
    from[(dice.unit() * from.len() as f32) as usize % from.len()]
}

/// One fork: its ways, which to take, what stands where, and the line naming it. The first fork
/// has two ways; later ones two or three. A line never repeats the kind of the one before.
fn fork(leg: usize, last: Option<u8>, dice: &mut Dice) -> Fork {
    let ways: Vec<Dir> = if leg > 0 && dice.chance(0.45) {
        vec![Dir::Left, Dir::Ahead, Dir::Right]
    } else {
        choose(
            &[
                [Dir::Left, Dir::Right],
                [Dir::Left, Dir::Ahead],
                [Dir::Ahead, Dir::Right],
            ],
            dice,
        )
        .to_vec()
    };
    let count = ways.len();
    let kinds: Vec<u8> = (0..4).filter(|kind| Some(*kind) != last).collect();
    let kind = choose(&kinds, dice);
    let mut unused: Vec<Landmark> = Landmark::ALL.to_vec();
    let take = |dice: &mut Dice, unused: &mut Vec<Landmark>| {
        let index = (dice.unit() * unused.len() as f32) as usize % unused.len();
        unused.remove(index)
    };
    let mut landmarks: Vec<Placed> = Vec::new();
    let mut stream = None;
    let (right, clue) = match kind {
        0 => {
            let right = (dice.unit() * count as f32) as usize % count;
            let landmark = take(dice, &mut unused);
            landmarks.push(Placed {
                landmark,
                spot: beside_only(right, count, dice),
            });
            // Every other way passes something else, and one of them perhaps its look-alike.
            let mut lookalike = landmark.lookalike().filter(|_| dice.chance(0.6));
            for way in (0..count).filter(|way| *way != right) {
                let other = match lookalike.take() {
                    Some(alike) if unused.contains(&alike) => {
                        unused.retain(|l| *l != alike);
                        alike
                    }
                    _ => take(dice, &mut unused),
                };
                landmarks.push(Placed {
                    landmark: other,
                    spot: beside_only(way, count, dice),
                });
            }
            (right, Clue::Past(landmark))
        }
        1 => {
            let side = if dice.chance(0.5) {
                Side::Left
            } else {
                Side::Right
            };
            let landmark = take(dice, &mut unused);
            landmarks.push(Placed {
                landmark,
                spot: Spot::Fork,
            });
            decorate(&mut landmarks, count, &mut unused, dice);
            let right = match side {
                Side::Left => 0,
                Side::Right => count - 1,
            };
            (right, Clue::Turn(side, landmark))
        }
        2 => {
            let right = (dice.unit() * count as f32) as usize % count;
            stream = Some(right);
            decorate(&mut landmarks, count, &mut unused, dice);
            (right, Clue::OverStream)
        }
        _ => {
            let right = (dice.unit() * count as f32) as usize % count;
            let (left, beyond) = (take(dice, &mut unused), take(dice, &mut unused));
            landmarks.push(Placed {
                landmark: left,
                spot: Spot::Beside(right),
            });
            landmarks.push(Placed {
                landmark: beyond,
                spot: Spot::Beside(right + 1),
            });
            // Something far down each other way.
            for way in (0..count).filter(|way| *way != right) {
                if dice.chance(0.7) {
                    landmarks.push(Placed {
                        landmark: take(dice, &mut unused),
                        spot: Spot::Far(way),
                    });
                }
            }
            (right, Clue::Between(left, beyond))
        }
    };
    let faded = match clue {
        Clue::OverStream => None,
        Clue::Past(_) => dice.chance(0.3).then_some(0),
        Clue::Turn(..) | Clue::Between(..) => {
            dice.chance(0.3).then(|| usize::from(dice.chance(0.5)))
        }
    };
    Fork {
        ways,
        right,
        stream,
        landmarks,
        clue,
        faded,
    }
}

/// Somewhere beside one way only: far off down it, or, for the leftmost or rightmost way, on its
/// outside.
fn beside_only(way: usize, count: usize, dice: &mut Dice) -> Spot {
    let outside = if way == 0 {
        Some(Spot::Beside(0))
    } else if way == count - 1 {
        Some(Spot::Beside(count))
    } else {
        None
    };
    match outside {
        Some(spot) if dice.chance(0.6) => spot,
        _ => Spot::Far(way),
    }
}

/// A landmark or two far off down the ways, for the look of the place: never one a line names.
fn decorate(
    landmarks: &mut Vec<Placed>,
    count: usize,
    unused: &mut Vec<Landmark>,
    dice: &mut Dice,
) {
    for way in 0..count {
        if dice.chance(0.55) && !unused.is_empty() {
            let index = (dice.unit() * unused.len() as f32) as usize % unused.len();
            landmarks.push(Placed {
                landmark: unused.remove(index),
                spot: Spot::Far(way),
            });
        }
    }
}

/// The dig: four places near the milestone, one the map means, one the same paces off by
/// something else, one by the same thing at other paces, and one neither.
fn dig(dice: &mut Dice) -> Dig {
    let mut paces = PACES.to_vec();
    let mut features = Feature::ALL.to_vec();
    let mut take_pace = |dice: &mut Dice| {
        let index = (dice.unit() * paces.len() as f32) as usize % paces.len();
        paces.remove(index)
    };
    let (right_paces, other_paces, third_paces) =
        (take_pace(dice), take_pace(dice), take_pace(dice));
    let mut take_feature = |dice: &mut Dice| {
        let index = (dice.unit() * features.len() as f32) as usize % features.len();
        features.remove(index)
    };
    let (right_by, other_by, third_by) =
        (take_feature(dice), take_feature(dice), take_feature(dice));
    let mut kinds = [
        (right_paces, right_by),
        (right_paces, other_by),
        (other_paces, right_by),
        (third_paces, third_by),
    ];
    // Shuffled among the four bearings.
    for index in (1..kinds.len()).rev() {
        let other = (dice.unit() * (index + 1) as f32) as usize % (index + 1);
        kinds.swap(index, other);
    }
    let spots: Vec<DigSpot> = kinds
        .iter()
        .zip(BEARINGS)
        .map(|(&(paces, feature), bearing)| DigSpot {
            paces,
            feature,
            at: (
                MILESTONE.0 + bearing.cos() * paces as f32 * PACE * 1.6,
                MILESTONE.1 + bearing.sin() * paces as f32 * PACE * 0.45 + 12.0,
            ),
        })
        .collect();
    let right = spots
        .iter()
        .position(|spot| spot.paces == right_paces && spot.feature == right_by)
        .unwrap_or(0);
    let faded = dice.chance(0.3).then(|| usize::from(dice.chance(0.5)));
    Dig {
        spots,
        right,
        faded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_line_read_in_full_names_one_way_and_it_is_the_right_one() {
        for map in 0..2000 {
            let route = Route::of(map);
            assert!((3..=4).contains(&route.forks.len()));
            for (leg, fork) in route.forks.iter().enumerate() {
                assert_eq!(
                    fork.matching(fork.clue),
                    vec![fork.right],
                    "map {map}, fork {leg}: {:?}",
                    fork
                );
                assert!(fork.ways.len() >= 2 && fork.right < fork.ways.len());
                assert!(fork.might_be(true) == vec![fork.right]);
                assert!(
                    fork.might_be(false).contains(&fork.right),
                    "a faded word hides the right way altogether"
                );
                let landmarks: BTreeSet<Landmark> = fork
                    .landmarks
                    .iter()
                    .map(|placed| placed.landmark)
                    .collect();
                assert_eq!(
                    landmarks.len(),
                    fork.landmarks.len(),
                    "something stands twice"
                );
            }
            assert_eq!(route.dig.might_be(true), vec![route.dig.right]);
            assert!(route.dig.might_be(false).len() <= 2);
        }
    }

    #[test]
    fn every_wrong_way_leads_somewhere_and_there_is_always_one_to_take_wrongly() {
        for map in 0..500 {
            for fork in Route::of(map).forks {
                let wrong: Vec<usize> = (0..fork.ways.len()).filter(|w| *w != fork.right).collect();
                assert!(!wrong.is_empty(), "map {map} has a fork with only one way");
            }
        }
    }

    #[test]
    fn the_same_map_always_sketches_the_same_route_and_every_map_a_different_one() {
        assert_eq!(Route::of(7), Route::of(7));
        let lines: BTreeSet<Vec<String>> = (0..200).map(|map| Route::of(map).lines(true)).collect();
        assert!(
            lines.len() > 190,
            "only {} different routes in 200",
            lines.len()
        );
        let kinds: BTreeSet<u8> = (0..200)
            .flat_map(|map| Route::of(map).forks.into_iter().map(|fork| kind(fork.clue)))
            .collect();
        assert_eq!(kinds.len(), 4, "every kind of line turns up");
        for map in 0..200 {
            let forks = Route::of(map).forks;
            for pair in forks.windows(2) {
                assert_ne!(
                    kind(pair[0].clue),
                    kind(pair[1].clue),
                    "map {map} repeats itself"
                );
            }
        }
    }

    #[test]
    fn a_faded_word_shows_faded_until_someone_reads_it() {
        let route = (0..400)
            .map(Route::of)
            .find(|route| route.forks.iter().any(|fork| fork.faded.is_some()))
            .expect("no map has a faded word");
        let fork = route
            .forks
            .iter()
            .find(|fork| fork.faded.is_some())
            .unwrap();
        assert!(
            fork.line(false).contains('\u{2026}'),
            "{}",
            fork.line(false)
        );
        assert!(!fork.line(true).contains('\u{2026}'));
        let lines = route.lines(false);
        assert_eq!(lines.len(), route.forks.len() + 1);
        assert!(lines.last().unwrap().ends_with("Dig!"));
    }

    #[test]
    fn look_alikes_stand_near_what_they_could_be_mistaken_for() {
        let alike = (0..400).any(|map| {
            Route::of(map).forks.iter().any(|fork| {
                let Clue::Past(sought) = fork.clue else {
                    return false;
                };
                fork.landmarks
                    .iter()
                    .any(|placed| Some(placed.landmark) == sought.lookalike())
            })
        });
        assert!(alike, "no map ever tests telling a split oak from an oak");
    }

    #[test]
    fn the_dig_spots_lie_their_paces_from_the_milestone() {
        for map in 0..200 {
            let dig = Route::of(map).dig;
            let mut far: Vec<(u32, f32)> = dig
                .spots
                .iter()
                .map(|spot| {
                    let (dx, dy) = (
                        (spot.at.0 - MILESTONE.0) / 1.6,
                        (spot.at.1 - 12.0 - MILESTONE.1) / 0.45,
                    );
                    (spot.paces, (dx * dx + dy * dy).sqrt())
                })
                .collect();
            far.sort_by(|a, b| a.1.total_cmp(&b.1));
            assert!(
                far.windows(2).all(|pair| pair[0].0 <= pair[1].0),
                "map {map}: further spots are fewer paces off"
            );
        }
    }
}
