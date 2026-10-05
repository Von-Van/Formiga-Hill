//! The Fairground, down the lane from the green: free play among the stalls, and games the colony
//! plays among themselves while the person watches. (Games the person plays, for finds to take
//! home, belong to the Woods.) The person chooses a game and, at most, who plays it: hide-and-seek,
//! the sack race, the high striker, hoopla, or the tug-of-war, whose sides the person may pick.

pub mod gear;
mod hide_and_seek;
pub mod high_striker;
pub mod hoopla;
pub mod prizes;
pub mod sack_race;
mod scenery;
pub mod tug_of_war;

use crate::cast::{Cast, Id};
use crate::daylight::Nightlights;
use crate::hilltop::Arrangement;
use crate::playground::{Layout, Patch, Playground};
use high_striker::HighStriker;
use hoopla::Hoopla;
use sack_race::SackRace;
use scenery::GROUND;
use std::sync::LazyLock;
use tug_of_war::{Side, TugOfWar};

pub use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
pub use hide_and_seek::{Event, HideAndSeek, Phase};

const SPOTS: [(&str, (f32, f32)); 4] = [
    ("centre", (200.0, 152.0)),
    ("front", (192.0, 198.0)),
    ("left", (56.0, 176.0)),
    ("right", (318.0, 186.0)),
];

/// The ground just behind each prop, where whoever stands is hidden: free play keeps out of it,
/// so nobody vanishes unless they mean to.
static FOOTPRINTS: LazyLock<Vec<Patch>> = LazyLock::new(|| {
    scenery::props()
        .0
        .iter()
        .map(|prop| {
            let (left, _, right, _) = prop.bounds();
            (
                left as f32 - 6.0,
                prop.base - 16.0,
                right as f32 + 6.0,
                prop.base + 3.0,
            )
        })
        .collect()
});

fn walkable(x: f32, y: f32) -> bool {
    let (left, top, right, bottom) = GROUND;
    let inside = |(l, t, r, b): Patch| x >= l && x <= r && y >= t && y <= b;
    inside((left, top, right, bottom)) && !FOOTPRINTS.iter().any(|patch| inside(*patch))
}

pub fn layout() -> Layout {
    Layout {
        walkable,
        ground: GROUND,
        spots: &SPOTS,
        seats: None,
        shade: None,
        // Up the lane from the green, in at the front right.
        entrance: (400.0, 196.0),
        entrance_step: (18.0, 5.0),
    }
}

/// A stretch of ground for a line of companions to stand along, from one end to the other: points
/// walked straight between.
pub type Path = &'static [(f32, f32)];

/// How much room a line of companions keeps between each one and the next.
pub const LINE_GAP: f32 = 3.0;

/// Where each of a line of companions stands along `paths`, in order, given how wide and how tall
/// each is: the first just inside the start, and each next one far enough along that nobody stands
/// over anyone else, side by side or one in front of another, however big they are, with `gap`
/// between, and nobody over the end. When a path is full the line goes on along the next; anyone
/// there is no room for at all stands a rank behind, between two places there, so nobody is ever
/// left without somewhere to be.
pub fn line_up(paths: &[Path], sizes: &[(f32, f32)], gap: f32) -> Vec<(f32, f32)> {
    let mut line: Vec<(f32, f32)> = Vec::with_capacity(sizes.len());
    let mut stood: Vec<((f32, f32), (f32, f32))> = Vec::new();
    let (mut path, mut along, mut last_half) = (0, 0.0, None::<f32>);
    for &(width, height) in sizes {
        let size = (width.max(1.0), height.max(1.0));
        let half = size.0 / 2.0;
        while let Some(&points) = paths.get(path) {
            let mut at = last_half.map_or(half, |last| along + last + gap + half);
            // On round a bend, a little further, until it is clear of everyone.
            let clear = |point: (f32, f32)| {
                stood
                    .iter()
                    .all(|&(other, other_size)| apart((point, size), (other, other_size), gap))
            };
            let mut found = None;
            while at + half <= length(points) {
                match point_along(points, at) {
                    Some(point) if clear(point) => {
                        found = Some(point);
                        break;
                    }
                    Some(_) => at += 1.0,
                    None => break,
                }
            }
            if let Some(point) = found {
                line.push(point);
                stood.push((point, size));
                (along, last_half) = (at, Some(half));
                break;
            }
            (path, along, last_half) = (path + 1, 0.0, None);
        }
        if paths.get(path).is_none() {
            break;
        }
    }
    // Anyone left over stands a rank behind, between two in the line.
    let placed = line.len();
    for extra in 0..sizes.len() - placed {
        let rank = extra as f32 + 1.0;
        let spot = match placed {
            0 => paths
                .first()
                .and_then(|path| path.first())
                .map_or((0.0, 0.0), |&(x, y)| (x + 8.0 * rank, y - 9.0 * rank)),
            1 => (line[0].0 + 8.0 * rank, line[0].1 - 9.0),
            _ => {
                let between = extra % (placed - 1);
                let rank = 1 + extra / (placed - 1);
                let (a, b) = (line[between], line[between + 1]);
                ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0 - 9.0 * rank as f32)
            }
        };
        line.push(spot);
    }
    line
}

/// How wide and tall each of `ids` stands, for lining them up.
pub fn sizes(ground: &Playground, ids: &[Id]) -> Vec<(f32, f32)> {
    ids.iter()
        .map(|id| {
            (
                ground.width(*id).unwrap_or(24.0),
                ground.height(*id).unwrap_or(24.0),
            )
        })
        .collect()
}

/// Whether two companions standing with their feet at these points, this wide and tall, are
/// drawn clear of each other by `gap`: side by side, or the one in front wholly below the other.
fn apart(
    ((ax, ay), (aw, ah)): ((f32, f32), (f32, f32)),
    ((bx, by), (bw, bh)): ((f32, f32), (f32, f32)),
    gap: f32,
) -> bool {
    let side_by_side = (ax - bx).abs() >= (aw + bw) / 2.0 + gap;
    let in_front = if ay >= by {
        ay - ah >= by + gap
    } else {
        by - bh >= ay + gap
    };
    side_by_side || in_front
}

/// How long `path` is, end to end.
fn length(path: &[(f32, f32)]) -> f32 {
    path.windows(2)
        .map(|pair| ((pair[1].0 - pair[0].0).powi(2) + (pair[1].1 - pair[0].1).powi(2)).sqrt())
        .sum()
}

/// The point `distance` along `path`, if the path is that long.
fn point_along(path: &[(f32, f32)], distance: f32) -> Option<(f32, f32)> {
    let mut left = distance;
    let first = *path.first()?;
    if left <= 0.0 {
        return Some(first);
    }
    for pair in path.windows(2) {
        let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
        let long = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        if left <= long && long > 0.0 {
            let t = left / long;
            return Some((x0 + (x1 - x0) * t, y0 + (y1 - y0) * t));
        }
        left -= long;
    }
    None
}

/// Shows what stands on the Hilltop, in silhouette on the skyline.
pub fn show_hilltop(ground: &mut Playground, hilltop: &Arrangement) {
    ground.set_backdrop(scenery::backdrop(hilltop));
}

/// The fairground with the colony walking in, and its games ready to play.
pub fn open(cast: &Cast, now: f32) -> (Playground, Games) {
    let (props, places) = scenery::props();
    let backdrop = scenery::backdrop(&Arrangement::new());
    let nightlights = Nightlights {
        lamps: scenery::lamplight(),
        sky: scenery::night_sky(&backdrop),
        indoors: false,
    };
    let mut ground = Playground::new(cast, now, layout(), backdrop, scenery::foreground(), props);
    ground.set_nightlights(nightlights);
    let games = Games {
        hide_and_seek: HideAndSeek::new(places),
        sack_race: SackRace::new(),
        high_striker: HighStriker::new(),
        hoopla: Hoopla::new(),
        tug_of_war: TugOfWar::new(),
    };
    games.set_out(&mut ground, now);
    (ground, games)
}

/// A game the colony plays here. Each says how the person picks who plays: one to be "it", or
/// any number of players, and what happens if nobody is picked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Game {
    HideAndSeek,
    SackRace,
    HighStriker,
    Hoopla,
    TugOfWar,
}

/// How the person says who plays a game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Players {
    /// One is picked to be "it"; everyone else plays. Nobody picked: whoever's turn it is, or
    /// whoever's keenest.
    It,
    /// Any number are picked, at least `least`; nobody picked is everyone.
    Some { least: usize },
    /// The person puts each on a side, or leaves it out; nobody put anywhere is everyone, sorted
    /// into sides by the colony itself.
    Sides,
}

impl Game {
    pub const ALL: [Self; 5] = [
        Self::HideAndSeek,
        Self::SackRace,
        Self::HighStriker,
        Self::Hoopla,
        Self::TugOfWar,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::HideAndSeek => "Hide-and-seek",
            Self::SackRace => "The sack race",
            Self::HighStriker => "The high striker",
            Self::Hoopla => "Hoopla",
            Self::TugOfWar => "The tug-of-war",
        }
    }

    pub fn players(self) -> Players {
        match self {
            Self::HideAndSeek => Players::It,
            Self::SackRace => Players::Some { least: 2 },
            Self::HighStriker | Self::Hoopla => Players::Some { least: 1 },
            Self::TugOfWar => Players::Sides,
        }
    }

    /// What the colony keeps for the display case the first time a game of it is seen through.
    pub fn souvenir(self) -> &'static str {
        match self {
            Self::HideAndSeek => crate::story::souvenirs::FAIR_TICKET,
            Self::SackRace => crate::story::souvenirs::RACE_ROSETTE,
            Self::HighStriker => crate::story::souvenirs::STRIKER_BELL,
            Self::Hoopla => crate::story::souvenirs::HOOPLA_TEDDY,
            Self::TugOfWar => crate::story::souvenirs::TUG_ROPE,
        }
    }
}

/// Who the person has picked to play each game, if anyone in particular, and the sides it has
/// put each on for the tug-of-war.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picks {
    picked: Vec<(Game, Vec<Id>)>,
    sides: Vec<(Id, Side)>,
}

impl Picks {
    /// Who is picked for `game`: nobody means the game's own choice.
    pub fn of(&self, game: Game) -> &[Id] {
        self.picked
            .iter()
            .find(|(picked, _)| *picked == game)
            .map_or(&[], |(_, ids)| ids.as_slice())
    }

    fn entry(&mut self, game: Game) -> &mut Vec<Id> {
        if let Some(index) = self.picked.iter().position(|(picked, _)| *picked == game) {
            return &mut self.picked[index].1;
        }
        self.picked.push((game, Vec::new()));
        let last = self.picked.len() - 1;
        &mut self.picked[last].1
    }

    /// Picks `id` for `game`, or unpicks it. Only one can be "it".
    pub fn toggle(&mut self, game: Game, id: Id) {
        let picked = self.entry(game);
        if let Some(at) = picked.iter().position(|own| *own == id) {
            picked.remove(at);
        } else if game.players() == Players::It {
            *picked = vec![id];
        } else {
            picked.push(id);
        }
    }

    /// Back to the game's own choice.
    pub fn clear(&mut self, game: Game) {
        self.entry(game).clear();
    }

    /// Whether enough are picked to play, with `travellers` here: at least as many as the game
    /// needs, or nobody, if there are that many here to play; for the tug-of-war, someone on each
    /// side, or nobody, if there are two here to sort into sides.
    pub fn enough(&self, game: Game, travellers: usize) -> bool {
        let picked = self.of(game).len();
        match game.players() {
            Players::It => true,
            Players::Some { least } => picked >= least || (picked == 0 && travellers >= least),
            Players::Sides => {
                let (left, right) = self.sides();
                (!left.is_empty() && !right.is_empty())
                    || (left.is_empty() && right.is_empty() && travellers >= 2)
            }
        }
    }

    /// The side `id` has been put on for the tug-of-war, if any.
    pub fn side(&self, id: Id) -> Option<Side> {
        self.sides
            .iter()
            .find(|(picked, _)| *picked == id)
            .map(|(_, side)| *side)
    }

    /// Puts `id` on a side for the tug-of-war, or leaves it out.
    pub fn put(&mut self, id: Id, side: Option<Side>) {
        self.sides.retain(|(picked, _)| *picked != id);
        if let Some(side) = side {
            self.sides.push((id, side));
        }
    }

    /// The sides the person has picked for the tug-of-war, in the order it picked them: both
    /// empty if it has left the colony to sort itself.
    pub fn sides(&self) -> (Vec<Id>, Vec<Id>) {
        let on = |wanted: Side| {
            self.sides
                .iter()
                .filter(|(_, side)| *side == wanted)
                .map(|(id, _)| *id)
                .collect()
        };
        (on(Side::Left), on(Side::Right))
    }

    /// Back to the colony sorting itself into sides.
    pub fn unsort(&mut self) {
        self.sides.clear();
    }

    /// Takes a pick of "it" once a game has started with it: next time, it is whoever's turn.
    pub fn used(&mut self, game: Game) {
        if game.players() == Players::It {
            self.clear(game);
        }
    }
}

/// The Fairground's games. One is played at a time.
pub struct Games {
    pub hide_and_seek: HideAndSeek,
    pub sack_race: SackRace,
    pub high_striker: HighStriker,
    pub hoopla: Hoopla,
    pub tug_of_war: TugOfWar,
}

impl Games {
    /// The game being played, if any.
    pub fn playing(&self) -> Option<Game> {
        if self.hide_and_seek.phase() != Phase::Ready {
            Some(Game::HideAndSeek)
        } else if self.sack_race.phase() != sack_race::Phase::Ready {
            Some(Game::SackRace)
        } else if self.high_striker.phase() != high_striker::Phase::Ready {
            Some(Game::HighStriker)
        } else if self.hoopla.phase() != hoopla::Phase::Ready {
            Some(Game::Hoopla)
        } else if self.tug_of_war.phase() != tug_of_war::Phase::Ready {
            Some(Game::TugOfWar)
        } else {
            None
        }
    }

    /// Starts `game` with whoever is picked, or the game's own choice if nobody is.
    pub fn start(
        &mut self,
        game: Game,
        ground: &mut Playground,
        cast: &Cast,
        picked: &[Id],
        now: f32,
    ) {
        if self.playing().is_some() {
            return;
        }
        match game {
            Game::HideAndSeek => self
                .hide_and_seek
                .start(ground, picked.first().copied(), now),
            Game::SackRace => self.sack_race.start(ground, cast, picked, now),
            Game::HighStriker => self.high_striker.start(ground, picked, now),
            Game::Hoopla => self.hoopla.start(ground, cast, picked, now),
            Game::TugOfWar => self.tug_of_war.start(ground, cast, picked, None, now),
        }
    }

    /// Starts the tug-of-war with the sides the person picked, or, if it picked none, everyone as
    /// the colony sorts itself.
    pub fn start_tug(
        &mut self,
        ground: &mut Playground,
        cast: &Cast,
        (left, right): (Vec<Id>, Vec<Id>),
        now: f32,
    ) {
        if self.playing().is_some() {
            return;
        }
        let sides = (!left.is_empty() && !right.is_empty()).then_some((left, right));
        self.tug_of_war.start(ground, cast, &[], sides, now);
    }

    /// Plays whichever game is on, and sets out its gear.
    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        self.hide_and_seek.tick(ground, now);
        self.sack_race.tick(ground, now);
        self.high_striker.tick(ground, now);
        self.hoopla.tick(ground, now);
        self.tug_of_war.tick(ground, now);
        self.set_out(ground, now);
    }

    /// The games' gear, where it is: the striker's bell, puck and mallet and the hoopla's pegs,
    /// prizes and rings always, the race's chalk line, posts and ribbon while a race is on, and the
    /// tug-of-war's rope and hay while a bout is; and whatever has been won at hoopla, carried by
    /// whoever won it.
    fn set_out(&self, ground: &mut Playground, now: f32) {
        let mut gear = self.high_striker.gear(ground, now);
        gear.extend(self.sack_race.gear(now));
        gear.extend(self.hoopla.gear(ground, now));
        gear.extend(self.tug_of_war.gear(ground, now));
        ground.set_fliers(gear);
        ground.set_fixtures(vec![self.hoopla.counter(now)]);
        ground.carry(self.hoopla.carried());
    }

    /// Calls off whatever game is on.
    pub fn stop(&mut self, ground: &mut Playground, now: f32) {
        match self.playing() {
            Some(Game::HideAndSeek) => self.hide_and_seek.stop(ground, now),
            Some(Game::SackRace) => self.sack_race.stop(ground, now),
            Some(Game::HighStriker) => self.high_striker.stop(ground, now),
            Some(Game::Hoopla) => self.hoopla.stop(ground, now),
            Some(Game::TugOfWar) => self.tug_of_war.stop(ground, now),
            None => {}
        }
    }

    /// Who wears a name tag all game, so the person can follow them, and what it says after the
    /// name: "it" at hide-and-seek, whoever is in front in a race, whoever is having a go at the
    /// striker or throwing at hoopla.
    pub fn tags(&self) -> Vec<(Id, &'static str)> {
        match self.playing() {
            Some(Game::HideAndSeek) => self
                .hide_and_seek
                .seeker()
                .map(|id| (id, "it"))
                .into_iter()
                .collect(),
            Some(Game::SackRace) => self
                .sack_race
                .leader()
                .map(|id| (id, "in front"))
                .into_iter()
                .collect(),
            Some(Game::HighStriker) => self
                .high_striker
                .player()
                .map(|(id, _)| (id, "having a go"))
                .into_iter()
                .collect(),
            Some(Game::Hoopla) => self
                .hoopla
                .player()
                .map(|id| (id, "throwing"))
                .into_iter()
                .collect(),
            // Everyone is in the thick of it: nobody in particular to follow.
            Some(Game::TugOfWar) | None => Vec::new(),
        }
    }
}

/// Colonies to play the games with in tests.
#[cfg(test)]
pub mod testing {
    use crate::cast::Cast;
    use formiga_travel::TravelerId;

    /// Desktop's sample colony grown to `count`, as big as a trip brings, by more of the same
    /// coming along under new names.
    pub fn colony_of(count: usize) -> Cast {
        let mut snapshot = formiga_travel::sample::snapshot();
        let originals = snapshot.travelers.clone();
        for extra in 0..count.saturating_sub(originals.len()) {
            let mut traveler = originals[extra % originals.len()].clone();
            traveler.id = TravelerId(traveler.id.0 ^ (0x5eed_0000 + extra as u64));
            traveler.name = format!("{} {}", traveler.name, extra + 2);
            snapshot.travelers.push(traveler);
        }
        snapshot.travelers.truncate(count);
        Cast::new(snapshot).unwrap()
    }

    /// How much two drawn rectangles, inclusive, overlap: in pixels.
    pub fn overlap(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> i32 {
        let across = (a.2.min(b.2) - a.0.max(b.0) + 1).max(0);
        let down = (a.3.min(b.3) - a.1.max(b.1) + 1).max(0);
        across * down
    }

    /// How many pixels a drawn rectangle, inclusive, covers.
    pub fn area(a: (i32, i32, i32, i32)) -> i32 {
        (a.2 - a.0 + 1).max(0) * (a.3 - a.1 + 1).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_keeps_everyone_a_little_apart_and_inside_its_ends() {
        let path: Path = &[(10.0, 50.0), (110.0, 50.0)];
        let places = line_up(&[path], &[(20.0, 9.0), (30.0, 9.0), (10.0, 9.0)], 4.0);
        assert_eq!(places, [(20.0, 50.0), (49.0, 50.0), (73.0, 50.0)]);
        // Round a corner, the next stands far enough on that it is clear of the last.
        let corner: Path = &[(30.0, 0.0), (0.0, 0.0), (0.0, 100.0)];
        let places = line_up(&[corner], &[(20.0, 30.0), (20.0, 30.0)], 2.0);
        let (first, second) = (places[0], places[1]);
        assert!(
            (first.0 - second.0).abs() >= 22.0 || second.1 - 30.0 >= first.1,
            "{first:?} and {second:?} stand over each other"
        );
        // A second path takes whoever the first has no room for, and anyone there is no room for
        // at all stands a rank behind, between two in the line.
        let short: Path = &[(0.0, 0.0), (40.0, 0.0)];
        let other: Path = &[(0.0, 20.0), (25.0, 20.0)];
        let places = line_up(&[short, other], &[(20.0, 9.0); 4], 4.0);
        assert_eq!(&places[..2], [(10.0, 0.0), (10.0, 20.0)]);
        assert_eq!(places.len(), 4, "nobody is left without somewhere to be");
        assert!(places[2].1 < 20.0 && places[3].1 < 20.0);
    }

    #[test]
    fn spots_and_the_way_in_are_clear_of_the_props() {
        for (name, (x, y)) in SPOTS {
            assert!(walkable(x, y), "{name} is behind something");
        }
        let (props, places) = scenery::props();
        for (prop, place) in props.iter().zip(&places) {
            let (x, y) = place.behind;
            assert!(
                !walkable(x, y),
                "free play would wander behind {}",
                place.name
            );
            assert!(prop.base < GROUND.3);
        }
    }

    #[test]
    fn free_play_never_hides_anyone() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let (mut ground, _) = open(&cast, 0.0);
        // Walking past behind something is fine; stopping there is not.
        let mut hidden_for = std::collections::HashMap::new();
        let mut now = 0.0;
        while now < 90.0 {
            now += 1.0 / 30.0;
            ground.tick(&cast, now);
            for id in ground.ids() {
                let (x, y) = ground.position(id).unwrap();
                let hidden = FOOTPRINTS
                    .iter()
                    .any(|&(l, t, r, b)| x > l + 6.0 && x < r - 6.0 && y > t + 4.0 && y < b - 3.0);
                let seconds = hidden_for.entry(id).or_insert(0.0);
                *seconds = if hidden { *seconds + 1.0 / 30.0 } else { 0.0 };
                assert!(*seconds < 3.0, "{id} stopped behind a prop at {now}s");
            }
        }
    }

    #[test]
    fn each_game_keeps_its_own_choice_of_who_plays() {
        let mut picks = Picks::default();
        for game in Game::ALL {
            assert!(
                picks.of(game).is_empty() && picks.enough(game, 6),
                "{game:?}"
            );
        }
        picks.toggle(Game::SackRace, 7);
        assert!(!picks.enough(Game::SackRace, 6), "a race needs two or more");
        picks.toggle(Game::SackRace, 9);
        assert!(picks.enough(Game::SackRace, 6));
        picks.toggle(Game::HighStriker, 7);
        assert!(
            picks.enough(Game::HighStriker, 6),
            "one can have a go on its own"
        );
        assert_eq!(picks.of(Game::SackRace), [7, 9]);
        assert_eq!(picks.of(Game::HighStriker), [7]);
        assert!(picks.of(Game::HideAndSeek).is_empty());
        picks.toggle(Game::SackRace, 7);
        assert_eq!(picks.of(Game::SackRace), [9], "picked again is unpicked");
        picks.clear(Game::SackRace);
        assert!(picks.of(Game::SackRace).is_empty());
    }

    #[test]
    fn the_person_can_put_each_on_a_side_of_the_rope_and_it_takes_someone_on_each() {
        let mut picks = Picks::default();
        assert!(
            picks.enough(Game::TugOfWar, 6),
            "nobody picked: the colony sorts itself"
        );
        picks.put(7, Some(Side::Left));
        assert!(!picks.enough(Game::TugOfWar, 6), "nobody to pull against");
        picks.put(9, Some(Side::Right));
        picks.put(11, Some(Side::Right));
        assert!(picks.enough(Game::TugOfWar, 6));
        assert_eq!(picks.sides(), (vec![7], vec![9, 11]));
        assert_eq!(picks.side(9), Some(Side::Right));
        picks.put(9, Some(Side::Left));
        picks.put(11, None);
        assert_eq!(picks.sides(), (vec![7, 9], Vec::new()));
        assert!(!picks.enough(Game::TugOfWar, 6));
        // The sides are the tug-of-war's own: nobody is picked for any other game by them.
        assert!(Game::ALL.iter().all(|game| picks.of(*game).is_empty()));
        picks.unsort();
        assert_eq!(picks.sides(), (Vec::new(), Vec::new()));
        assert!(picks.enough(Game::TugOfWar, 6));
        assert!(
            !picks.enough(Game::TugOfWar, 1),
            "one traveller has nobody to pull against"
        );
        assert!(!picks.enough(Game::SackRace, 1), "nor anyone to race");
    }

    #[test]
    fn the_tug_of_war_is_pulled_with_the_persons_own_sides() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let (mut ground, mut games) = open(&cast, 0.0);
        let ids: Vec<Id> = cast.ids().collect();
        let sides = (vec![ids[1]], vec![ids[3], ids[4]]);
        games.start_tug(&mut ground, &cast, sides, 0.0);
        assert_eq!(games.playing(), Some(Game::TugOfWar));
        let (mut left, mut right) = games.tug_of_war.sides();
        left.sort_unstable();
        right.sort_unstable();
        let mut wanted = vec![ids[3], ids[4]];
        wanted.sort_unstable();
        assert_eq!((left, right), (vec![ids[1]], wanted));
        games.stop(&mut ground, 1.0);
        // With no sides picked, everyone, as the colony sorts itself.
        games.start_tug(&mut ground, &cast, (Vec::new(), Vec::new()), 1.0);
        let (left, right) = games.tug_of_war.sides();
        assert_eq!(left.len() + right.len(), cast.members.len());
    }

    #[test]
    fn only_one_is_it_and_it_is_whoevers_turn_after_that() {
        let mut picks = Picks::default();
        picks.toggle(Game::HideAndSeek, 7);
        picks.toggle(Game::HideAndSeek, 9);
        assert_eq!(picks.of(Game::HideAndSeek), [9]);
        picks.toggle(Game::SackRace, 7);
        picks.toggle(Game::SackRace, 9);
        picks.used(Game::HideAndSeek);
        picks.used(Game::SackRace);
        assert!(picks.of(Game::HideAndSeek).is_empty());
        assert_eq!(picks.of(Game::SackRace), [7, 9], "racers stay picked");
    }

    #[test]
    fn one_game_at_a_time_and_each_ends_and_hands_everyone_back() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let (mut ground, mut games) = open(&cast, 0.0);
        let mut now = 0.0;
        for game in Game::ALL {
            assert_eq!(games.playing(), None);
            games.start(game, &mut ground, &cast, &[], now);
            assert_eq!(games.playing(), Some(game));
            for other in Game::ALL {
                games.start(other, &mut ground, &cast, &[], now);
                assert_eq!(
                    games.playing(),
                    Some(game),
                    "{other:?} started over {game:?}"
                );
            }
            let until = now + 300.0;
            while games.playing().is_some() && now < until {
                now += 1.0 / 30.0;
                ground.tick(&cast, now);
                games.tick(&mut ground, now);
            }
            assert_eq!(games.playing(), None, "{game:?} never ended");
        }
    }

    #[test]
    fn whoever_is_in_the_thick_of_a_game_wears_a_name_tag() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let (mut ground, mut games) = open(&cast, 0.0);
        assert!(games.tags().is_empty());
        games.start(Game::HighStriker, &mut ground, &cast, &[], 0.0);
        let tags = games.tags();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].1, "having a go");
        games.stop(&mut ground, 1.0);
        games.start(
            Game::HideAndSeek,
            &mut ground,
            &cast,
            &[cast.members[2].id],
            1.0,
        );
        assert_eq!(games.tags(), vec![(cast.members[2].id, "it")]);
    }

    #[test]
    fn the_race_course_is_set_out_only_for_a_race_and_the_striker_always_stands() {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let (mut ground, mut games) = open(&cast, 0.0);
        let striker_only = games.high_striker.gear(&mut ground, 0.0).len();
        assert!(games.sack_race.gear(0.0).is_empty());
        games.start(Game::SackRace, &mut ground, &cast, &[], 0.0);
        assert!(!games.sack_race.gear(0.0).is_empty());
        assert_eq!(striker_only, 3, "the bell, the puck and the mallet");
    }

    #[test]
    fn whatever_is_won_at_hoopla_is_carried_about_the_fairground() {
        let mut snapshot = formiga_travel::sample::snapshot();
        for traveler in &mut snapshot.travelers {
            traveler.character.axes.impulsiveness = 0.0;
        }
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut games) = open(&cast, 0.0);
        let mut now = 0.0;
        for _ in 0..5 {
            games.start(Game::Hoopla, &mut ground, &cast, &[], now);
            while games.playing().is_some() && now < 2000.0 {
                now += 1.0 / 30.0;
                ground.tick(&cast, now);
                games.tick(&mut ground, now);
            }
            if !games.hoopla.carried().is_empty() {
                break;
            }
        }
        let (carrier, _) = *games.hoopla.carried().first().expect("nobody won a thing");
        // Drawn with it, and with it again however often the fairground goes on after.
        for _ in 0..3 {
            now += 1.0 / 30.0;
            ground.tick(&cast, now);
            games.tick(&mut ground, now);
            let with = ground.bounds(carrier, now).unwrap();
            ground.carry(&[]);
            let without = ground.bounds(carrier, now).unwrap();
            assert!(with.1 < without.1, "its prize isn't drawn with it");
        }
    }

    #[test]
    fn every_game_keeps_its_own_souvenir_and_none_is_a_storys() {
        let mut kept: Vec<&str> = Game::ALL.iter().map(|game| game.souvenir()).collect();
        kept.dedup();
        assert_eq!(kept.len(), Game::ALL.len());
        for souvenir in kept {
            assert!(crate::story::souvenirs::name(souvenir).is_some());
            assert!(!crate::story::souvenirs::a_story_can_give(souvenir));
        }
    }
}
