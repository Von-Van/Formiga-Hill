//! Who each traveller is, turned into what they do.
//!
//! Nothing here is written for a particular creature. Everything is read off what the snapshot
//! carried: the nine temperament axes and the kind they lean to, the habits picked up on the
//! desktop, family, and how each pair gets on. So two colonies doing the same thing at the Hill
//! look different without anyone authoring a branch for either of them.

use crate::cast::{Id, Member};
use crate::dice::Dice;
use formiga_art::{BodyClip, ExpressionKind};
use formiga_core::{ActionKind, Axes, Celebration, Gesture, Habit, HabitCue, TemperamentKind};

/// One beat of a performance: what the body does, the face it wears, and for how long.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beat {
    pub clip: BodyClip,
    pub expression: ExpressionKind,
    pub seconds: f32,
    /// Hold the clip's first frame rather than playing it: a snack held up and looked over.
    pub held: bool,
    /// Turn about on the spot as it goes: a twirl, or circling before a nap.
    pub spin: bool,
    /// A little sign over its head while the beat lasts.
    pub cue: Option<Cue>,
    /// Free play may cut this beat short to start something else.
    pub idle: bool,
}

impl Beat {
    pub fn new(clip: impl Into<BodyClip>, expression: ExpressionKind, seconds: f32) -> Self {
        Self {
            clip: clip.into(),
            expression,
            seconds,
            held: false,
            spin: false,
            cue: None,
            idle: false,
        }
    }

    fn cue(mut self, cue: Cue) -> Self {
        self.cue = Some(cue);
        self
    }

    fn held(mut self) -> Self {
        self.held = true;
        self
    }

    fn spin(mut self) -> Self {
        self.spin = true;
        self
    }

    fn idle(mut self) -> Self {
        self.idle = true;
        self
    }
}

/// Signs drawn over a creature's head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cue {
    Heart,
    Exclaim,
    Note,
    Huff,
    Sparkle,
    Sleep,
    /// A speech bubble while its line is on show.
    Speech,
}

/// Something the person at the Hill holds out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Offer {
    Pet,
    Snack,
    Toy,
}

/// What a traveller sets out to do next, on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Idea {
    Wander,
    Rest,
    Watch,
    Sit,
    Nap,
    ShowOff,
    Visit(Id),
    PlayWith(Id),
    FollowParent(Id),
}

/// Who is about that this traveller might go to.
#[derive(Clone, Copy, Debug, Default)]
pub struct Company {
    /// The closest free friend, if any.
    pub friend: Option<Id>,
    /// A free friend it plays with.
    pub playmate: Option<Id>,
    /// Its parent, if it is a mini and its parent came.
    pub parent: Option<Id>,
}

/// One traveller's character, read once from the snapshot.
#[derive(Clone, Debug)]
pub struct Character {
    pub kind: TemperamentKind,
    pub axes: Axes,
    /// Its own tempo, from Desktop: how lively its walk is.
    pub pace: f32,
    pub habits: Vec<Habit>,
    pub parent: Option<Id>,
    pub celebration: Celebration,
}

impl Character {
    pub fn of(member: &Member) -> Self {
        let character = &member.traveler.character;
        Self {
            kind: character.temperament.into(),
            axes: character.axes.into(),
            pace: member.traveler.motion.activity,
            habits: member
                .traveler
                .habits
                .iter()
                .map(|habit| (*habit).into())
                .collect(),
            parent: member.parent(),
            // Desktop's own choice for the real companion; worked out the same way from the
            // stand-in only if a snapshot somehow lacks it.
            celebration: member
                .traveler
                .motion
                .celebration
                .map(Into::into)
                .unwrap_or_else(|| Celebration::for_creature(&member.creature)),
        }
    }

    /// Strolling pace, in scene pixels a second.
    pub fn walk_speed(&self) -> f32 {
        let pace = 13.0 + self.pace * 15.0;
        if self.parent.is_some() {
            pace - 2.0
        } else {
            pace
        }
    }

    /// The face it wears doing nothing in particular.
    pub fn idle_face(&self) -> ExpressionKind {
        match self.kind {
            TemperamentKind::Grump => ExpressionKind::Grumpy,
            TemperamentKind::Showoff | TemperamentKind::Troublemaker => ExpressionKind::Smug,
            TemperamentKind::Scholar => ExpressionKind::Focused,
            TemperamentKind::Explorer | TemperamentKind::Oddball => ExpressionKind::Curious,
            TemperamentKind::Lazybones => ExpressionKind::Sleepy,
            TemperamentKind::Guardian => ExpressionKind::Determined,
            TemperamentKind::Wallflower => ExpressionKind::Neutral,
            TemperamentKind::Sweetheart => ExpressionKind::Content,
        }
    }

    /// The face it walks with.
    pub fn walk_face(&self) -> ExpressionKind {
        if self.axes.playfulness > 0.65 {
            ExpressionKind::Joy
        } else if self.axes.curiosity > 0.6 {
            ExpressionKind::Curious
        } else {
            ExpressionKind::Content
        }
    }

    /// Its celebration, as beats.
    pub fn celebrate(&self, seconds: f32) -> Beat {
        let beat = match self.celebration {
            Celebration::Hop => Beat::new(Gesture::Cheer, ExpressionKind::Joy, seconds),
            Celebration::Dance => Beat::new(Gesture::Bop, ExpressionKind::Joy, seconds),
            Celebration::Twirl => Beat::new(Gesture::Cheer, ExpressionKind::Joy, seconds).spin(),
        };
        beat.cue(Cue::Sparkle)
    }

    /// The flourish a habit adds at the start of its kind of moment, as Desktop draws it.
    pub fn flourish(&self, cue: HabitCue) -> Option<Beat> {
        let habit = self
            .habits
            .iter()
            .copied()
            .find(|habit| habit.cue() == cue)?;
        let seconds = habit.seconds();
        Some(match habit {
            Habit::LooksFoodOver => {
                Beat::new(ActionKind::Eat, ExpressionKind::Curious, seconds).held()
            }
            Habit::StretchesBeforeNaps => {
                Beat::new(Gesture::Stretch, ExpressionKind::Content, seconds)
            }
            Habit::CirclesBeforeNaps => {
                Beat::new(ActionKind::Traverse, ExpressionKind::Content, seconds).spin()
            }
            Habit::WavesHello => Beat::new(Gesture::Reach, ExpressionKind::Joy, seconds),
            Habit::PlayBows => Beat::new(Gesture::Crouch, ExpressionKind::Joy, seconds),
        })
    }

    /// How it answers something offered. `trust` is how much it has warmed to the person over this
    /// visit, from 0; `in_a_row` how many times running it has just been offered the same thing.
    pub fn react(&self, offer: Offer, trust: f32, in_a_row: u32, dice: &mut Dice) -> Vec<Beat> {
        let a = self.axes;
        // Wariness eases as the visit goes on: nothing here punishes, but shy creatures take a
        // little while to come round.
        let wary = (a.suspicion - trust).max(0.0);
        let shy = wary > 0.55 && a.boldness < 0.5;
        let mut beats = Vec::new();
        match offer {
            Offer::Pet => {
                if in_a_row >= 3 && a.feistiness > 0.55 {
                    // Enough is enough, for now.
                    beats
                        .push(Beat::new(Gesture::Huff, ExpressionKind::Grumpy, 1.4).cue(Cue::Huff));
                    return beats;
                }
                if shy {
                    beats.push(
                        Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.4).cue(Cue::Exclaim),
                    );
                    beats.push(Beat::new(Gesture::Peek, ExpressionKind::Worried, 1.1));
                    if a.affection > 0.4 {
                        beats.push(Beat::new(ActionKind::Idle, ExpressionKind::Content, 0.8));
                    }
                    return beats;
                }
                let warmth = a.affection * 0.65 + (1.0 - wary) * 0.35;
                if warmth > 0.6 {
                    beats.push(
                        Beat::new(ActionKind::PetReaction, ExpressionKind::Affectionate, 1.3)
                            .cue(Cue::Heart),
                    );
                } else {
                    beats.push(Beat::new(
                        ActionKind::PetReaction,
                        ExpressionKind::Content,
                        1.0,
                    ));
                }
                beats.push(match self.kind {
                    TemperamentKind::Grump => Beat::new(Gesture::Huff, ExpressionKind::Smug, 1.0),
                    TemperamentKind::Showoff => {
                        Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.2)
                    }
                    TemperamentKind::Sweetheart => {
                        Beat::new(Gesture::Beg, ExpressionKind::Pleading, 1.0).cue(Cue::Heart)
                    }
                    TemperamentKind::Lazybones => {
                        Beat::new(ActionKind::Idle, ExpressionKind::Sleepy, 1.2)
                    }
                    _ if warmth > 0.6 && a.energy > 0.55 => self.celebrate(0.9),
                    _ => Beat::new(ActionKind::Idle, ExpressionKind::Content, 0.8),
                });
            }
            Offer::Snack => {
                if let Some(flourish) = self.flourish(HabitCue::Meal) {
                    beats.push(flourish);
                } else if wary > 0.5 || (a.curiosity < 0.35 && a.suspicion > 0.5) {
                    beats.push(Beat::new(Gesture::Watch, ExpressionKind::Worried, 0.9));
                }
                if a.impulsiveness > 0.65 {
                    // Gone in a moment.
                    beats
                        .push(Beat::new(ActionKind::Eat, ExpressionKind::Joy, 1.4).cue(Cue::Heart));
                } else {
                    let seconds = 2.2 + (1.0 - a.energy) * 1.2 + dice.range(0.0, 0.4);
                    beats.push(Beat::new(ActionKind::Eat, ExpressionKind::Content, seconds));
                }
                beats.push(match self.kind {
                    TemperamentKind::Lazybones => {
                        Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 1.4)
                    }
                    TemperamentKind::Grump => {
                        Beat::new(ActionKind::Idle, ExpressionKind::Smug, 1.0)
                    }
                    _ if a.playfulness > 0.6 => self.celebrate(0.8),
                    _ => Beat::new(ActionKind::Idle, ExpressionKind::Content, 0.9),
                });
            }
            Offer::Toy => {
                if let Some(flourish) = self.flourish(HabitCue::Play) {
                    beats.push(flourish);
                }
                match self.kind {
                    TemperamentKind::Scholar => {
                        beats.push(Beat::new(Gesture::Watch, ExpressionKind::Focused, 1.2));
                        beats.push(Beat::new(
                            ActionKind::SoloPlay,
                            ExpressionKind::Focused,
                            1.8,
                        ));
                    }
                    TemperamentKind::Wallflower => {
                        beats.push(Beat::new(Gesture::Peek, ExpressionKind::Worried, 0.8));
                        beats.push(Beat::new(
                            ActionKind::SoloPlay,
                            ExpressionKind::Content,
                            2.0,
                        ));
                    }
                    _ if a.playfulness > 0.55 => {
                        let seconds = 2.6 + a.energy * 1.6;
                        beats.push(
                            Beat::new(ActionKind::SoloPlay, ExpressionKind::Joy, seconds)
                                .cue(Cue::Note),
                        );
                    }
                    _ if a.playfulness < 0.35 => {
                        beats.push(Beat::new(Gesture::Watch, ExpressionKind::Curious, 1.0));
                        beats.push(Beat::new(
                            ActionKind::SoloPlay,
                            ExpressionKind::Focused,
                            1.5,
                        ));
                        beats.push(Beat::new(ActionKind::Idle, ExpressionKind::Bored, 0.8));
                        return beats;
                    }
                    _ => beats.push(Beat::new(
                        ActionKind::SoloPlay,
                        ExpressionKind::Content,
                        2.4,
                    )),
                }
                beats.push(match self.kind {
                    TemperamentKind::Showoff => {
                        Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.2)
                    }
                    _ => self.celebrate(0.9),
                });
            }
        }
        beats
    }

    /// What it sets out to do next, given who is about.
    pub fn idea(&self, company: Company, dice: &mut Dice) -> Idea {
        let a = self.axes;
        let lazy = self.kind == TemperamentKind::Lazybones;
        let showy = self.kind == TemperamentKind::Showoff;
        let quiet = matches!(
            self.kind,
            TemperamentKind::Wallflower | TemperamentKind::Scholar
        );
        let mut ideas = vec![
            (Idea::Wander, 1.0 + a.energy + a.curiosity * 0.5),
            (Idea::Rest, 0.5 + (1.0 - a.energy)),
            (
                Idea::Watch,
                a.curiosity * 0.6 + if quiet { 0.5 } else { 0.0 },
            ),
            (Idea::Sit, 0.25 + if quiet || lazy { 0.5 } else { 0.0 }),
            (
                Idea::Nap,
                (1.0 - a.energy) * 0.4 + if lazy { 0.8 } else { 0.0 },
            ),
            (Idea::ShowOff, if showy { 0.8 } else { a.boldness * 0.15 }),
        ];
        if let Some(friend) = company.friend {
            ideas.push((Idea::Visit(friend), a.social * 1.4));
        }
        if let Some(playmate) = company.playmate {
            ideas.push((
                Idea::PlayWith(playmate),
                a.playfulness * (0.4 + a.social) * 1.5,
            ));
        }
        if let Some(parent) = company.parent {
            ideas.push((Idea::FollowParent(parent), 2.5));
        }
        let weights: Vec<f32> = ideas.iter().map(|(_, weight)| *weight).collect();
        dice.weighted(&weights)
            .map_or(Idea::Rest, |index| ideas[index].0)
    }

    /// The beats an idea plays out once it has got wherever it was going.
    pub fn beats_for(&self, idea: Idea, dice: &mut Dice) -> Vec<Beat> {
        let rest = 1.2 + (1.0 - self.axes.energy) * 3.0 + dice.range(0.0, 2.0);
        match idea {
            Idea::Wander | Idea::Rest | Idea::FollowParent(_) => {
                vec![Beat::new(ActionKind::Idle, self.idle_face(), rest).idle()]
            }
            Idea::Watch => {
                vec![Beat::new(Gesture::Watch, ExpressionKind::Curious, 1.5 + rest * 0.5).idle()]
            }
            Idea::Sit => vec![Beat::new(Gesture::Sit, ExpressionKind::Content, 2.5 + rest).idle()],
            Idea::Nap => {
                let mut beats: Vec<_> = self.flourish(HabitCue::Nap).into_iter().collect();
                beats.push(Beat::new(Gesture::Crouch, ExpressionKind::Sleepy, 0.45));
                beats.push(
                    Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 6.0 + rest)
                        .cue(Cue::Sleep)
                        .idle(),
                );
                beats
            }
            Idea::ShowOff => vec![
                Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.6).cue(Cue::Sparkle),
                Beat::new(ActionKind::Idle, ExpressionKind::Smug, 1.0).idle(),
            ],
            Idea::Visit(_) => {
                let mut beats: Vec<_> = self.flourish(HabitCue::Hello).into_iter().collect();
                beats.push(Beat::new(
                    ActionKind::Idle,
                    ExpressionKind::Joy,
                    2.0 + rest * 0.5,
                ));
                beats
            }
            Idea::PlayWith(_) => self.play_together(),
        }
    }

    /// Playing with a friend: a play bow if it has one, the game, and its own celebration.
    pub fn play_together(&self) -> Vec<Beat> {
        let mut beats: Vec<_> = self.flourish(HabitCue::Play).into_iter().collect();
        beats.push(
            Beat::new(
                ActionKind::SocialPlay,
                ExpressionKind::Joy,
                3.2 + self.axes.energy,
            )
            .cue(Cue::Note),
        );
        beats.push(self.celebrate(0.9));
        beats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    fn with(kind: TemperamentKind, tune: impl FnOnce(&mut Axes)) -> Character {
        let mut character = Character::of(&sample().members[0]);
        let mut axes = Axes::MIDDLING;
        tune(&mut axes);
        character.kind = kind;
        character.axes = axes;
        character.pace = axes.energy;
        character.habits.clear();
        character.parent = None;
        character
    }

    fn faces(beats: &[Beat]) -> Vec<ExpressionKind> {
        beats.iter().map(|beat| beat.expression).collect()
    }

    #[test]
    fn an_affectionate_creature_and_a_wary_one_take_a_pat_differently() {
        let mut dice = Dice::new(1);
        let fond = with(TemperamentKind::Sweetheart, |a| {
            a.affection = 0.9;
            a.suspicion = 0.1;
        });
        let wary = with(TemperamentKind::Wallflower, |a| {
            a.affection = 0.3;
            a.suspicion = 0.9;
            a.boldness = 0.2;
        });
        let fond_beats = fond.react(Offer::Pet, 0.0, 0, &mut dice);
        let wary_beats = wary.react(Offer::Pet, 0.0, 0, &mut dice);
        assert_eq!(fond_beats[0].cue, Some(Cue::Heart));
        assert_eq!(wary_beats[0].cue, Some(Cue::Exclaim));
        assert_ne!(faces(&fond_beats), faces(&wary_beats));
    }

    #[test]
    fn a_wary_creature_comes_round_over_the_visit() {
        let mut dice = Dice::new(2);
        let wary = with(TemperamentKind::Wallflower, |a| {
            a.affection = 0.7;
            a.suspicion = 0.8;
            a.boldness = 0.2;
        });
        let first = wary.react(Offer::Pet, 0.0, 0, &mut dice);
        let later = wary.react(Offer::Pet, 0.5, 0, &mut dice);
        assert_eq!(first[0].expression, ExpressionKind::Startled);
        assert_eq!(later[0].clip, BodyClip::Action(ActionKind::PetReaction));
    }

    #[test]
    fn a_feisty_creature_has_had_enough_after_a_while_but_nothing_is_lost() {
        let mut dice = Dice::new(3);
        let feisty = with(TemperamentKind::Grump, |a| a.feistiness = 0.8);
        let fourth = feisty.react(Offer::Pet, 1.0, 3, &mut dice);
        assert_eq!(fourth.len(), 1);
        assert_eq!(fourth[0].cue, Some(Cue::Huff));
        let later = feisty.react(Offer::Pet, 1.0, 0, &mut dice);
        assert_eq!(later[0].clip, BodyClip::Action(ActionKind::PetReaction));
    }

    #[test]
    fn habits_from_the_desktop_open_their_moments() {
        let mut dice = Dice::new(4);
        let mut careful = with(TemperamentKind::Scholar, |_| {});
        careful.habits = vec![Habit::LooksFoodOver];
        let snack = careful.react(Offer::Snack, 0.0, 0, &mut dice);
        assert!(snack[0].held && snack[0].clip == BodyClip::Action(ActionKind::Eat));

        let mut bower = with(TemperamentKind::Troublemaker, |a| a.playfulness = 0.9);
        bower.habits = vec![Habit::PlayBows];
        let toy = bower.react(Offer::Toy, 0.0, 0, &mut dice);
        assert_eq!(toy[0].clip, BodyClip::Gesture(Gesture::Crouch));
        assert_eq!(
            bower.play_together()[0].clip,
            BodyClip::Gesture(Gesture::Crouch)
        );
    }

    #[test]
    fn a_playful_creature_plays_longer_and_celebrates_where_a_serious_one_tires() {
        let mut dice = Dice::new(5);
        let playful = with(TemperamentKind::Troublemaker, |a| {
            a.playfulness = 0.9;
            a.energy = 0.9;
        });
        let serious = with(TemperamentKind::Guardian, |a| a.playfulness = 0.1);
        let fun = playful.react(Offer::Toy, 0.0, 0, &mut dice);
        let dull = serious.react(Offer::Toy, 0.0, 0, &mut dice);
        assert_eq!(fun.last().unwrap().cue, Some(Cue::Sparkle));
        assert_eq!(dull.last().unwrap().expression, ExpressionKind::Bored);
    }

    #[test]
    fn energy_and_kind_shape_what_it_does_with_itself() {
        let lazy = with(TemperamentKind::Lazybones, |a| a.energy = 0.1);
        let lively = with(TemperamentKind::Explorer, |a| {
            a.energy = 0.95;
            a.curiosity = 0.9;
        });
        let count = |character: &Character, wanted: Idea| {
            let mut dice = Dice::new(6);
            (0..500)
                .filter(|_| character.idea(Company::default(), &mut dice) == wanted)
                .count()
        };
        assert!(count(&lazy, Idea::Nap) > count(&lively, Idea::Nap) * 3);
        assert!(count(&lively, Idea::Wander) > count(&lazy, Idea::Wander));
        assert!(lively.walk_speed() > lazy.walk_speed());
    }

    #[test]
    fn friends_playmates_and_rivals_come_from_the_bonds() {
        use formiga_travel::Band;
        let cast = sample();
        let ids: Vec<_> = cast.ids().collect();
        let mut friendships = 0;
        for &id in &ids {
            if let Some(friend) = cast.closest_friend(id, ids.clone()) {
                friendships += 1;
                let bond = cast.bond(id, friend).unwrap();
                assert!(bond.warmth >= Band::Medium && bond.friction < bond.warmth);
                // Nobody else is warmer to it.
                for &other in &ids {
                    if let Some(other_bond) = cast.bond(id, other)
                        && other_bond.friction < other_bond.warmth
                    {
                        assert!(other_bond.warmth <= bond.warmth);
                    }
                }
            }
            if let Some(mate) = cast.playmate(id, ids.clone()) {
                assert!(cast.bond(id, mate).unwrap().playfulness >= Band::Medium);
            }
            for &other in &ids {
                if cast.at_odds(id, other) {
                    let bond = cast.bond(id, other).unwrap();
                    assert!(bond.friction > bond.warmth);
                }
            }
        }
        assert!(friendships > 0, "someone in the sample has a friend");
    }

    #[test]
    fn minis_mostly_stay_by_their_parents() {
        let mut mini = with(TemperamentKind::Explorer, |_| {});
        mini.parent = Some(1);
        let mut dice = Dice::new(7);
        let company = Company {
            parent: Some(1),
            ..Company::default()
        };
        let following = (0..400)
            .filter(|_| mini.idea(company, &mut dice) == Idea::FollowParent(1))
            .count();
        assert!(following > 150, "only followed {following} times in 400");
    }
}
