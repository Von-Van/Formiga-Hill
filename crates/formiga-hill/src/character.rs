//! Who each traveller is, turned into what they do.
//!
//! Nothing here is written for a particular creature. Everything is read off what the snapshot
//! carried: the nine temperament axes and the kind they lean to, the habits picked up on the
//! desktop, family, and how each pair gets on. So two colonies doing the same thing at the Hill
//! look different without anyone authoring a branch for either of them.

use crate::cast::{Id, Member};
use crate::dice::Dice;
use crate::finds::Use;
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
    /// A brush: held on a companion and stroked to groom it (see `Character::brushed`).
    Brush,
}

/// How far a grooming has got.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Brushing {
    Begun,
    Halfway,
    /// Groomed till it shines.
    Done,
}

/// How a companion lends a hand when the colony builds something on the Hilltop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Knack {
    /// Off to fetch things and back with them.
    Fetching,
    /// Knocking it all together and stamping it down.
    Hammering,
    /// Holding things steady while the others work.
    Steadying,
    /// Standing back and pointing out where everything goes.
    Directing,
    /// Supervising, sitting down. Mostly asleep.
    Supervising,
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
    /// How big it is against an average adult, from Desktop: its stature times its share of an
    /// adult's size, so a little one is small and a tall one tall.
    pub size: f32,
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
            size: f32::from(member.traveler.stature_percent) / 100.0
                * f32::from(member.traveler.scale_percent)
                / 100.0,
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
            Offer::Brush => {
                // Shown the brush, before any stroking: a look at what it is.
                beats.push(if shy {
                    Beat::new(Gesture::Peek, ExpressionKind::Worried, 0.9)
                } else {
                    Beat::new(Gesture::Watch, ExpressionKind::Curious, 0.8)
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

    /// How it takes being groomed, as the brushing gets under way, halfway through, and done:
    /// its own way, from its temperament. While being brushed it keeps still, held in a pose
    /// that lasts until the brushing stops.
    pub fn brushed(&self, stage: Brushing, trust: f32) -> Vec<Beat> {
        const HELD: f32 = 600.0;
        let a = self.axes;
        let wary = (a.suspicion - trust).max(0.0) > 0.55 && a.boldness < 0.5;
        match stage {
            Brushing::Begun => vec![match self.kind {
                TemperamentKind::Lazybones => {
                    Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, HELD).cue(Cue::Sleep)
                }
                _ if wary => Beat::new(Gesture::Worry, ExpressionKind::Worried, HELD),
                TemperamentKind::Grump => Beat::new(Gesture::Huff, ExpressionKind::Grumpy, HELD),
                _ if a.playfulness > 0.7 => Beat::new(Gesture::Bop, ExpressionKind::Joy, HELD),
                _ => Beat::new(ActionKind::PetReaction, ExpressionKind::Affectionate, HELD),
            }],
            Brushing::Halfway => vec![match self.kind {
                TemperamentKind::Lazybones => {
                    Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, HELD).cue(Cue::Sleep)
                }
                // Coming round to it after all.
                TemperamentKind::Grump => {
                    Beat::new(ActionKind::PetReaction, ExpressionKind::Content, HELD)
                }
                TemperamentKind::Sweetheart => {
                    Beat::new(Gesture::Beg, ExpressionKind::Pleading, HELD).cue(Cue::Heart)
                }
                TemperamentKind::Showoff => Beat::new(Gesture::Strut, ExpressionKind::Smug, HELD),
                TemperamentKind::Scholar => {
                    Beat::new(Gesture::Watch, ExpressionKind::Focused, HELD)
                }
                _ if wary => Beat::new(ActionKind::PetReaction, ExpressionKind::Content, HELD),
                _ => Beat::new(ActionKind::PetReaction, ExpressionKind::Affectionate, HELD)
                    .cue(Cue::Heart),
            }],
            Brushing::Done => match self.kind {
                TemperamentKind::Lazybones => {
                    vec![Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 3.0).cue(Cue::Sleep)]
                }
                TemperamentKind::Showoff => {
                    vec![Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.6).cue(Cue::Sparkle)]
                }
                TemperamentKind::Grump => {
                    vec![Beat::new(Gesture::Huff, ExpressionKind::Smug, 1.2).cue(Cue::Sparkle)]
                }
                _ => vec![self.celebrate(1.2).cue(Cue::Sparkle)],
            },
        }
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
    /// Enjoying something that stands on the ground, in its own way.
    pub fn enjoy(&self, use_: Use, dice: &mut Dice) -> Vec<Beat> {
        let lazy = self.kind == TemperamentKind::Lazybones;
        let linger = dice.range(0.0, 1.5);
        match use_ {
            Use::Look => {
                let face = match self.kind {
                    TemperamentKind::Scholar => ExpressionKind::Focused,
                    TemperamentKind::Grump => ExpressionKind::Grumpy,
                    _ => ExpressionKind::Curious,
                };
                vec![Beat::new(Gesture::Watch, face, 1.8 + linger).idle()]
            }
            Use::Sit => vec![Beat::new(Gesture::Sit, ExpressionKind::Content, 3.0 + linger).idle()],
            Use::Gaze if lazy => vec![
                Beat::new(Gesture::Watch, ExpressionKind::Sleepy, 1.5),
                Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 4.0 + linger)
                    .cue(Cue::Sleep)
                    .idle(),
            ],
            Use::Gaze => {
                vec![Beat::new(Gesture::Watch, ExpressionKind::Focused, 2.5 + linger).idle()]
            }
            Use::Play => vec![
                self.celebrate(1.2),
                Beat::new(ActionKind::SoloPlay, ExpressionKind::Joy, 1.5 + linger).idle(),
            ],
            Use::Rest if lazy || dice.chance(0.3) => vec![
                Beat::new(Gesture::Crouch, ExpressionKind::Sleepy, 0.45),
                Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 4.0 + linger)
                    .cue(Cue::Sleep)
                    .idle(),
            ],
            Use::Rest => {
                vec![Beat::new(Gesture::Sit, ExpressionKind::Content, 3.0 + linger).idle()]
            }
            Use::Tend => self.tend(linger),
        }
    }

    /// Looking after something growing, its own way: getting down low to sniff it, patting the
    /// leaves, dancing round it, or sitting by it and keeping it company. It grows just the same
    /// whoever tends it, and whether anyone does.
    fn tend(&self, linger: f32) -> Vec<Beat> {
        let a = self.axes;
        match self.kind {
            TemperamentKind::Lazybones => vec![
                Beat::new(Gesture::Sit, ExpressionKind::Content, 1.5),
                Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 4.0 + linger)
                    .cue(Cue::Sleep)
                    .idle(),
            ],
            TemperamentKind::Scholar => vec![
                Beat::new(Gesture::Crouch, ExpressionKind::Focused, 1.6),
                Beat::new(Gesture::Watch, ExpressionKind::Focused, 1.8 + linger).idle(),
            ],
            TemperamentKind::Grump => vec![
                Beat::new(Gesture::Crouch, ExpressionKind::Grumpy, 1.2),
                Beat::new(Gesture::Reach, ExpressionKind::Content, 1.0),
                Beat::new(Gesture::Huff, ExpressionKind::Smug, 1.0).idle(),
            ],
            _ if a.affection > 0.6 => vec![
                Beat::new(Gesture::Reach, ExpressionKind::Affectionate, 1.4).cue(Cue::Heart),
                Beat::new(Gesture::Sit, ExpressionKind::Content, 2.5 + linger).idle(),
            ],
            _ if a.playfulness > 0.65 => vec![
                Beat::new(Gesture::Bop, ExpressionKind::Joy, 1.6).cue(Cue::Note),
                Beat::new(ActionKind::Idle, ExpressionKind::Joy, 1.0 + linger).idle(),
            ],
            _ if a.curiosity > 0.55 => vec![
                Beat::new(Gesture::Crouch, ExpressionKind::Curious, 1.4),
                Beat::new(Gesture::Watch, ExpressionKind::Curious, 1.5 + linger).idle(),
            ],
            _ => vec![
                Beat::new(Gesture::Crouch, ExpressionKind::Content, 1.2),
                Beat::new(Gesture::Sit, ExpressionKind::Content, 2.5 + linger).idle(),
            ],
        }
    }

    /// How it lends a hand building something: a lazybones supervises, a scholar directs, a little
    /// one fetches, and the rest as their energy, feistiness and affection take them.
    pub fn knack(&self) -> Knack {
        let a = self.axes;
        match self.kind {
            _ if self.parent.is_some() => Knack::Fetching,
            TemperamentKind::Lazybones => Knack::Supervising,
            TemperamentKind::Scholar => Knack::Directing,
            TemperamentKind::Guardian => Knack::Steadying,
            TemperamentKind::Grump | TemperamentKind::Troublemaker => Knack::Hammering,
            TemperamentKind::Explorer => Knack::Fetching,
            _ if a.energy > 0.6 => Knack::Fetching,
            _ if a.feistiness > 0.5 => Knack::Hammering,
            _ if a.affection > 0.55 => Knack::Steadying,
            _ if a.boldness > 0.6 => Knack::Directing,
            _ => Knack::Fetching,
        }
    }

    /// Its work at the building, once there, for as long as the building lasts: each beat goes on
    /// until the work is done and something else is asked of it. A fetcher's trips to and fro are
    /// the scene's; this is what it does with what it carried.
    pub fn work(&self, knack: Knack) -> Vec<Beat> {
        const UNTIL_DONE: f32 = 600.0;
        match knack {
            Knack::Hammering => {
                let mut beats = Vec::new();
                for _ in 0..4 {
                    beats.push(Beat::new(Gesture::Stomp, ExpressionKind::Determined, 0.7));
                    beats.push(Beat::new(Gesture::Reach, ExpressionKind::Determined, 0.5));
                }
                beats.push(Beat::new(
                    Gesture::Stomp,
                    ExpressionKind::Determined,
                    UNTIL_DONE,
                ));
                beats
            }
            Knack::Steadying => vec![Beat::new(
                Gesture::Heave,
                ExpressionKind::Determined,
                UNTIL_DONE,
            )],
            Knack::Directing => vec![
                Beat::new(Gesture::Reach, ExpressionKind::Focused, 1.0),
                Beat::new(Gesture::Watch, ExpressionKind::Focused, 1.4),
                Beat::new(Gesture::Reach, ExpressionKind::Focused, 0.8),
                Beat::new(Gesture::Watch, ExpressionKind::Focused, UNTIL_DONE),
            ],
            Knack::Supervising => vec![
                Beat::new(Gesture::Sit, ExpressionKind::Content, 1.8),
                Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 1.0),
                Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, UNTIL_DONE).cue(Cue::Sleep),
            ],
            Knack::Fetching => vec![Beat::new(
                Gesture::Heave,
                ExpressionKind::Determined,
                UNTIL_DONE,
            )],
        }
    }

    /// Picking something up on a fetching trip.
    pub fn pick_up(&self) -> Beat {
        Beat::new(Gesture::Crouch, ExpressionKind::Determined, 0.6)
    }

    /// Greeting what it helped build, as it appears: its own celebration, a show-off's strut, and
    /// a supervisor woken by the puff.
    pub fn admire(&self, knack: Knack) -> Vec<Beat> {
        let mut beats = Vec::new();
        if knack == Knack::Supervising {
            beats.push(Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.5).cue(Cue::Exclaim));
        }
        beats.push(self.celebrate(1.4));
        if self.kind == TemperamentKind::Showoff {
            beats.push(Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.0));
        }
        beats.push(Beat::new(Gesture::Watch, ExpressionKind::Joy, 1.0).idle());
        beats
    }

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
    use crate::cast::sample;

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
    fn each_looks_after_something_growing_in_its_own_way() {
        let mut dice = Dice::new(8);
        let tenders = [
            with(TemperamentKind::Lazybones, |_| {}),
            with(TemperamentKind::Scholar, |_| {}),
            with(TemperamentKind::Sweetheart, |a| a.affection = 0.9),
            with(TemperamentKind::Troublemaker, |a| {
                a.affection = 0.2;
                a.playfulness = 0.9;
            }),
            with(TemperamentKind::Explorer, |a| {
                a.affection = 0.2;
                a.playfulness = 0.2;
                a.curiosity = 0.9;
            }),
        ];
        let mut ways = std::collections::BTreeSet::new();
        for tender in &tenders {
            let beats = tender.enjoy(Use::Tend, &mut dice);
            assert!(
                beats.last().is_some_and(|beat| beat.idle),
                "free to move on after"
            );
            ways.insert(format!("{:?}", faces(&beats)));
        }
        assert_eq!(ways.len(), tenders.len(), "some tend alike: {ways:?}");
    }

    #[test]
    fn everyone_lends_a_hand_building_in_its_own_way_until_it_is_done() {
        let builders = [
            with(TemperamentKind::Lazybones, |_| {}),
            with(TemperamentKind::Scholar, |_| {}),
            with(TemperamentKind::Guardian, |_| {}),
            with(TemperamentKind::Grump, |_| {}),
            with(TemperamentKind::Explorer, |_| {}),
        ];
        let knacks: std::collections::HashSet<Knack> =
            builders.iter().map(Character::knack).collect();
        assert_eq!(knacks.len(), builders.len(), "some build alike: {knacks:?}");
        for builder in &builders {
            let work = builder.work(builder.knack());
            let lasts: f32 = work.iter().map(|beat| beat.seconds).sum();
            assert!(
                lasts > 60.0,
                "{:?} stops before the building does",
                builder.kind
            );
            let admired = builder.admire(builder.knack());
            assert!(
                admired.last().is_some_and(|beat| beat.idle),
                "free again after"
            );
        }
        let mut little = with(TemperamentKind::Lazybones, |_| {});
        little.parent = Some(1);
        assert_eq!(little.knack(), Knack::Fetching, "a little one runs errands");
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
