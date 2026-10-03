//! The Fairground, down the lane from the green: free play among the stalls, and games the colony
//! plays among themselves while the person watches. (Games the person plays, for finds to take
//! home, belong to the Woods.) The person chooses a game and, at most, who plays it: hide-and-seek,
//! the sack race, or the high striker.

pub mod gear;
mod hide_and_seek;
pub mod high_striker;
pub mod sack_race;
mod scenery;

use crate::cast::{Cast, Id};
use crate::daylight::Nightlights;
use crate::hilltop::Arrangement;
use crate::playground::{Layout, Patch, Playground};
use high_striker::HighStriker;
use sack_race::SackRace;
use scenery::GROUND;
use std::sync::LazyLock;

pub use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
pub use hide_and_seek::{Event, HideAndSeek, Phase};

const SPOTS: [(&str, (f32, f32)); 4] = [
    ("centre", (200.0, 152.0)),
    ("front", (192.0, 198.0)),
    ("left", (56.0, 176.0)),
    ("right", (350.0, 190.0)),
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
}

/// How the person says who plays a game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Players {
    /// One is picked to be "it"; everyone else plays. Nobody picked: whoever's turn it is, or
    /// whoever's keenest.
    It,
    /// Any number are picked, at least `least`; nobody picked is everyone.
    Some { least: usize },
}

impl Game {
    pub const ALL: [Self; 3] = [Self::HideAndSeek, Self::SackRace, Self::HighStriker];

    pub fn name(self) -> &'static str {
        match self {
            Self::HideAndSeek => "Hide-and-seek",
            Self::SackRace => "The sack race",
            Self::HighStriker => "The high striker",
        }
    }

    pub fn players(self) -> Players {
        match self {
            Self::HideAndSeek => Players::It,
            Self::SackRace => Players::Some { least: 2 },
            Self::HighStriker => Players::Some { least: 1 },
        }
    }

    /// What the colony keeps for the display case the first time a game of it is seen through.
    pub fn souvenir(self) -> &'static str {
        match self {
            Self::HideAndSeek => crate::story::souvenirs::FAIR_TICKET,
            Self::SackRace => crate::story::souvenirs::RACE_ROSETTE,
            Self::HighStriker => crate::story::souvenirs::STRIKER_BELL,
        }
    }
}

/// Who the person has picked to play each game, if anyone in particular.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picks {
    picked: Vec<(Game, Vec<Id>)>,
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

    /// Whether enough are picked to play: nobody, or at least as many as the game needs.
    pub fn enough(&self, game: Game) -> bool {
        let picked = self.of(game).len();
        match game.players() {
            Players::It => true,
            Players::Some { least } => picked == 0 || picked >= least,
        }
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
        }
    }

    /// Plays whichever game is on, and sets out its gear.
    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        self.hide_and_seek.tick(ground, now);
        self.sack_race.tick(ground, now);
        self.high_striker.tick(ground, now);
        self.set_out(ground, now);
    }

    /// The games' gear, where it is: the striker's bell, puck and mallet always, and the race's
    /// chalk line, posts and ribbon while a race is on.
    fn set_out(&self, ground: &mut Playground, now: f32) {
        let mut gear = self.high_striker.gear(ground, now);
        gear.extend(self.sack_race.gear(now));
        ground.set_fliers(gear);
    }

    /// Calls off whatever game is on.
    pub fn stop(&mut self, ground: &mut Playground, now: f32) {
        match self.playing() {
            Some(Game::HideAndSeek) => self.hide_and_seek.stop(ground, now),
            Some(Game::SackRace) => self.sack_race.stop(ground, now),
            Some(Game::HighStriker) => self.high_striker.stop(ground, now),
            None => {}
        }
    }

    /// Who wears a name tag all game, so the person can follow them, and what it says after the
    /// name: "it" at hide-and-seek, whoever is in front in a race, whoever is having a go at the
    /// striker.
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
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert!(picks.of(game).is_empty() && picks.enough(game), "{game:?}");
        }
        picks.toggle(Game::SackRace, 7);
        assert!(!picks.enough(Game::SackRace), "a race needs two or more");
        picks.toggle(Game::SackRace, 9);
        assert!(picks.enough(Game::SackRace));
        picks.toggle(Game::HighStriker, 7);
        assert!(
            picks.enough(Game::HighStriker),
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
