//! Plays a story in the Clubhouse with the colony as its cast.
//!
//! The director works through a scene's beats in order. A beat starts once everyone busy with the
//! beats before it has finished, unless it is marked `meanwhile`, and never while anyone it is
//! about is still busy with a `meanwhile` beat; a line waits for the person to read it; a choice
//! waits for them to choose. Beats about someone who did not come, and lines
//! naming them, are passed over, so a story plays with whoever is there.

use super::casting::cast_roles;
use super::lines::fill;
use super::script::{Action, Condition, Feeling, Place, Pose, Story, Who};
use crate::actor::Step;
use crate::cast::{Cast, Id};
use crate::character::{Beat as ActorBeat, Cue};
use crate::playground::Playground;
use formiga_art::ExpressionKind;
use formiga_core::{ActionKind, Gesture};
use formiga_travel::Band;
use std::collections::BTreeSet;

/// A line on show: who says it, if anyone, and the words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shown {
    pub speaker: Option<(Id, String)>,
    pub text: String,
}

/// More than this many beats played in one go without anything to wait for is a loop.
const MAX_INSTANT_BEATS: usize = 1000;

pub struct Director {
    story: Story,
    players: Vec<Option<Id>>,
    scene: usize,
    beat: usize,
    flags: BTreeSet<String>,
    /// Everyone busy with the beats in hand, which the next beat waits for.
    busy: Vec<Id>,
    /// Everyone busy with a `meanwhile` beat, which only a beat about them waits for.
    background: Vec<Id>,
    /// Whether the beat being played is a `meanwhile` one.
    in_background: bool,
    waiting_until: f32,
    shown: Option<Shown>,
    choices: Vec<(String, usize)>,
    souvenirs: Vec<String>,
    finished: bool,
}

impl Director {
    pub fn new(story: Story, cast: &Cast, seed: u64) -> Result<Self, String> {
        let players = cast_roles(&story, cast, seed)?;
        let scene = story.start;
        Ok(Self {
            story,
            players,
            scene,
            beat: 0,
            flags: BTreeSet::new(),
            busy: Vec::new(),
            background: Vec::new(),
            in_background: false,
            waiting_until: 0.0,
            shown: None,
            choices: Vec::new(),
            souvenirs: Vec::new(),
            finished: false,
        })
    }

    pub fn title(&self) -> &str {
        &self.story.title
    }

    pub fn story_id(&self) -> &str {
        &self.story.id
    }

    /// Everyone with a part, who free play leaves alone until the story ends.
    pub fn players(&self) -> Vec<Id> {
        self.players.iter().flatten().copied().collect()
    }

    pub fn shown(&self) -> Option<&Shown> {
        self.shown.as_ref()
    }

    pub fn choices(&self) -> Vec<&str> {
        self.choices.iter().map(|(text, _)| text.as_str()).collect()
    }

    pub fn souvenirs(&self) -> &[String] {
        &self.souvenirs
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    /// The person has read the line on show.
    pub fn read_on(&mut self) {
        self.shown = None;
    }

    pub fn choose(&mut self, index: usize) {
        if let Some(&(_, scene)) = self.choices.get(index) {
            self.goto(scene);
            self.choices.clear();
        }
    }

    /// Plays as far as the story can go at `now`.
    pub fn run(&mut self, room: &mut Playground, cast: &Cast, now: f32) {
        for _ in 0..MAX_INSTANT_BEATS {
            if self.finished || self.shown.is_some() || !self.choices.is_empty() {
                return;
            }
            if now < self.waiting_until {
                return;
            }
            let Some(beat) = self.story.scenes[self.scene].beats.get(self.beat).cloned() else {
                // A scene that runs out without going anywhere is the end.
                self.finished = true;
                return;
            };
            // A beat follows the ones before it unless it is `meanwhile`. Either way it waits for
            // whatever is still going on in the background that involves anyone it is about, which
            // starting it would cut short.
            let involved = self.involved(&beat.action);
            let waiting = (!beat.meanwhile && self.busy.iter().any(|id| room.busy(*id)))
                || self
                    .background
                    .iter()
                    .any(|id| involved.contains(id) && room.busy(*id));
            if waiting {
                return;
            }
            if !beat.meanwhile {
                self.busy.clear();
            }
            self.background.retain(|id| room.busy(*id));
            self.in_background = beat.meanwhile;
            self.beat += 1;
            if beat
                .when
                .as_ref()
                .is_some_and(|when| !self.holds(when, cast))
            {
                continue;
            }
            self.play(&beat.action, room, cast, now);
        }
        // A story that never waits for anything has looped; end it rather than hang.
        self.finished = true;
    }

    fn goto(&mut self, scene: usize) {
        self.scene = scene;
        self.beat = 0;
    }

    fn player(&self, role: usize) -> Option<Id> {
        self.players.get(role).copied().flatten()
    }

    fn who(&self, who: Who) -> Vec<Id> {
        match who {
            Who::All => self.players(),
            Who::Role(role) => self.player(role).into_iter().collect(),
        }
    }

    fn name_of(&self, cast: &Cast, role_name: &str) -> Option<String> {
        let role = self
            .story
            .roles
            .iter()
            .position(|role| role.name == role_name)?;
        cast.member(self.player(role)?)
            .map(|member| member.name.clone())
    }

    fn line(&self, key: &str, cast: &Cast, voice: Option<Id>) -> Option<String> {
        let line = self.story.lines.get(key)?;
        let kind = voice
            .and_then(|id| cast.member(id))
            .map(|member| member.kind());
        fill(line.for_kind(kind), |name| self.name_of(cast, name))
    }

    fn perform(&mut self, room: &mut Playground, id: Id, steps: Vec<Step>, now: f32) {
        room.direct(id, steps, now);
        if self.in_background {
            self.background.push(id);
        } else {
            self.busy.push(id);
        }
    }

    /// Everyone a beat is about, whom it waits for if they are still busy in the background.
    fn involved(&self, action: &Action) -> Vec<Id> {
        match action {
            Action::Walk { who, .. }
            | Action::Face { who, .. }
            | Action::React { who, .. }
            | Action::Pose { who, .. }
            | Action::Eat { who }
            | Action::Nap { who }
            | Action::Celebrate { who } => self.who(*who),
            Action::Play { who, with } => [*who, *with]
                .iter()
                .filter_map(|role| self.player(*role))
                .collect(),
            Action::Say { who, .. } => self.player(*who).into_iter().collect(),
            Action::Narrate {
                about: Some(role), ..
            } => self.player(*role).into_iter().collect(),
            _ => Vec::new(),
        }
    }

    fn play(&mut self, action: &Action, room: &mut Playground, cast: &Cast, now: f32) {
        match action {
            Action::Walk { who, to } => {
                for (slot, id) in self.who(*who).into_iter().enumerate() {
                    let spot = match to {
                        Place::Named(place) => Some(room.spot(place, slot)),
                        Place::Beside(role) => {
                            self.player(*role).map(|other| room.beside_of(id, other))
                        }
                    };
                    if let Some(to) = spot {
                        self.perform(room, id, vec![Step::Stride { to }], now);
                    }
                }
            }
            Action::Face { who, to } => {
                if let Some(other) = self.player(*to) {
                    for id in self.who(*who) {
                        room.direct(id, vec![Step::Face(other)], now);
                    }
                }
            }
            Action::React { who, feeling } => {
                for id in self.who(*who) {
                    let beats = room
                        .character(id)
                        .map(|c| feel(c, *feeling))
                        .unwrap_or_default();
                    self.perform(room, id, beats.into_iter().map(Step::Beat).collect(), now);
                }
            }
            Action::Pose { who, pose } => {
                for id in self.who(*who) {
                    self.perform(room, id, vec![Step::Beat(strike(*pose))], now);
                }
            }
            Action::Eat { who } => {
                for id in self.who(*who) {
                    let mut beats: Vec<ActorBeat> = room
                        .character(id)
                        .and_then(|c| c.flourish(formiga_core::HabitCue::Meal))
                        .into_iter()
                        .collect();
                    beats.push(ActorBeat::new(
                        ActionKind::Eat,
                        ExpressionKind::Content,
                        2.4,
                    ));
                    self.perform(room, id, beats.into_iter().map(Step::Beat).collect(), now);
                }
            }
            Action::Nap { who } => {
                for id in self.who(*who) {
                    let mut beats: Vec<ActorBeat> = room
                        .character(id)
                        .and_then(|c| c.flourish(formiga_core::HabitCue::Nap))
                        .into_iter()
                        .collect();
                    beats.push(ActorBeat::new(
                        Gesture::Crouch,
                        ExpressionKind::Sleepy,
                        0.45,
                    ));
                    let mut sleep = ActorBeat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 4.0);
                    sleep.cue = Some(Cue::Sleep);
                    beats.push(sleep);
                    self.perform(room, id, beats.into_iter().map(Step::Beat).collect(), now);
                }
            }
            Action::Celebrate { who } => {
                for id in self.who(*who) {
                    if let Some(beat) = room.character(id).map(|c| c.celebrate(1.0)) {
                        self.perform(room, id, vec![Step::Beat(beat)], now);
                    }
                }
            }
            Action::Play { who, with } => {
                if let (Some(a), Some(b)) = (self.player(*who), self.player(*with)) {
                    for (id, other) in [(a, b), (b, a)] {
                        let beats = room
                            .character(id)
                            .map(|c| c.play_together())
                            .unwrap_or_default();
                        let steps = std::iter::once(Step::Face(other))
                            .chain(beats.into_iter().map(Step::Beat))
                            .collect();
                        self.perform(room, id, steps, now);
                    }
                }
            }
            Action::Say { who, line } => {
                let Some(speaker) = self.player(*who) else {
                    return;
                };
                if let Some(text) = self.line(line, cast, Some(speaker)) {
                    let name = cast
                        .member(speaker)
                        .map(|m| m.name.clone())
                        .unwrap_or_default();
                    self.shown = Some(Shown {
                        speaker: Some((speaker, name)),
                        text,
                    });
                }
            }
            Action::Narrate { line, about } => {
                let voice = match about {
                    Some(role) => match self.player(*role) {
                        Some(id) => Some(id),
                        None => return,
                    },
                    None => None,
                };
                if let Some(text) = self.line(line, cast, voice) {
                    self.shown = Some(Shown {
                        speaker: None,
                        text,
                    });
                }
            }
            Action::Wait { seconds } => self.waiting_until = now + seconds,
            Action::Choose { options } => {
                self.choices = options
                    .iter()
                    .filter_map(|(key, scene)| Some((self.line(key, cast, None)?, *scene)))
                    .collect();
            }
            Action::Goto { scene } => self.goto(*scene),
            Action::If { condition, goto } => {
                if self.holds(condition, cast) {
                    self.goto(*goto);
                }
            }
            Action::Set { flag } => {
                self.flags.insert(flag.clone());
            }
            Action::Souvenir { id } => {
                if !self.souvenirs.contains(id) {
                    self.souvenirs.push(id.clone());
                }
            }
            Action::End => self.finished = true,
        }
    }

    fn holds(&self, condition: &Condition, cast: &Cast) -> bool {
        let member = |role: &usize| self.player(*role).and_then(|id| cast.member(id));
        match condition {
            Condition::Flag(flag) => self.flags.contains(flag),
            Condition::Present(role) => self.player(*role).is_some(),
            Condition::Mini(role) => member(role).is_some_and(|m| m.parent().is_some()),
            Condition::Kind(role, kind) => member(role).is_some_and(|m| m.kind() == *kind),
            Condition::Trait(role, label) => member(role).is_some_and(|m| m.has_trait(label)),
            Condition::Habit(role, habit) => member(role).is_some_and(|m| m.has_habit(*habit)),
            Condition::High(role, axis) => member(role).is_some_and(|m| axis.of(&m.axes()) >= 0.65),
            Condition::Low(role, axis) => member(role).is_some_and(|m| axis.of(&m.axes()) <= 0.35),
            Condition::Close(a, b) => match (self.player(*a), self.player(*b)) {
                (Some(a), Some(b)) => cast
                    .bond(a, b)
                    .is_some_and(|bond| bond.warmth >= Band::High),
                _ => false,
            },
            Condition::Rivals(a, b) => match (self.player(*a), self.player(*b)) {
                (Some(a), Some(b)) => cast.at_odds(a, b),
                _ => false,
            },
            Condition::Not(inner) => !self.holds(inner, cast),
        }
    }
}

/// A feeling, as this particular creature shows it.
fn feel(character: &crate::character::Character, feeling: Feeling) -> Vec<ActorBeat> {
    let a = character.axes;
    let slow = if a.energy < 0.35 { 0.4 } else { 0.0 };
    let beat = |clip: Gesture, face, seconds: f32| ActorBeat::new(clip, face, seconds + slow);
    match feeling {
        Feeling::Joy if a.energy > 0.55 || a.impulsiveness > 0.65 => vec![character.celebrate(1.0)],
        Feeling::Joy => vec![ActorBeat::new(
            ActionKind::Idle,
            ExpressionKind::Joy,
            1.2 + slow,
        )],
        Feeling::Surprise => {
            let mut start = beat(Gesture::Gasp, ExpressionKind::Startled, 0.5);
            start.cue = Some(Cue::Exclaim);
            // The bold get over it quickly and want to know more.
            if a.boldness > 0.6 {
                vec![start, beat(Gesture::Watch, ExpressionKind::Curious, 0.9)]
            } else {
                vec![start, beat(Gesture::Worry, ExpressionKind::Worried, 0.9)]
            }
        }
        Feeling::Worry => vec![beat(Gesture::Worry, ExpressionKind::Worried, 1.4)],
        Feeling::Fond => {
            let mut fond =
                ActorBeat::new(ActionKind::PetReaction, ExpressionKind::Affectionate, 1.4);
            fond.cue = Some(Cue::Heart);
            vec![fond]
        }
        Feeling::Proud if a.boldness < 0.35 => {
            vec![ActorBeat::new(ActionKind::Idle, ExpressionKind::Smug, 1.2)]
        }
        Feeling::Proud => vec![beat(Gesture::Strut, ExpressionKind::Smug, 1.4)],
        Feeling::Sleepy => vec![beat(Gesture::Yawn, ExpressionKind::Yawning, 1.4)],
        Feeling::Grumpy => {
            let mut huff = beat(Gesture::Huff, ExpressionKind::Grumpy, 1.4);
            huff.cue = Some(Cue::Huff);
            vec![huff]
        }
        Feeling::Curious => vec![beat(Gesture::Watch, ExpressionKind::Curious, 1.4)],
        Feeling::Bored => vec![ActorBeat::new(
            ActionKind::Idle,
            ExpressionKind::Bored,
            1.4 + slow,
        )],
        Feeling::Shy => vec![beat(Gesture::Peek, ExpressionKind::Worried, 1.4)],
    }
}

/// A pose, struck and held for a moment.
fn strike(pose: Pose) -> ActorBeat {
    let (gesture, face, seconds) = match pose {
        Pose::Inspect => (Gesture::Watch, ExpressionKind::Focused, 1.6),
        Pose::Sit => (Gesture::Sit, ExpressionKind::Content, 3.0),
        Pose::Crouch => (Gesture::Crouch, ExpressionKind::Determined, 1.2),
        Pose::Stretch => (Gesture::Stretch, ExpressionKind::Content, 1.5),
        Pose::Wave => (Gesture::Reach, ExpressionKind::Joy, 1.2),
        Pose::Peek => (Gesture::Peek, ExpressionKind::Worried, 1.6),
        Pose::Strut => (Gesture::Strut, ExpressionKind::Smug, 1.6),
        Pose::Cheer => (Gesture::Cheer, ExpressionKind::Joy, 1.2),
        Pose::Dance => (Gesture::Bop, ExpressionKind::Joy, 1.6),
        Pose::Huff => (Gesture::Huff, ExpressionKind::Grumpy, 1.4),
        Pose::Beg => (Gesture::Beg, ExpressionKind::Pleading, 1.4),
        Pose::Watch => (Gesture::Watch, ExpressionKind::Curious, 1.6),
        Pose::Cover => (Gesture::Cover, ExpressionKind::Worried, 1.4),
        Pose::Balance => (Gesture::Balance, ExpressionKind::Determined, 1.6),
    };
    ActorBeat::new(gesture, face, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::lines::Lines;
    use crate::story::script::parse_story;

    #[test]
    fn a_meanwhile_beat_waits_for_anyone_still_busy_with_the_one_before() {
        let story = parse_story(
            r#"
            [story]
            id = "test"
            title = "title"
            area = "clubhouse"
            start = "one"
            [roles.host]
            select = ["any"]
            [[scenes]]
            id = "one"
            beats = [
              { walk = "host", to = "left", meanwhile = true },
              { react = "host", feeling = "joy", meanwhile = true },
            ]
            "#,
            &Lines::parse(r#"title = "Test""#).unwrap(),
            1,
        )
        .unwrap();
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        let mut room =
            crate::clubhouse::Clubhouse::open(&cast, 0.0, &Default::default(), Vec::new());
        let mut director = Director::new(story, &cast, 1).unwrap();
        let host = director.players()[0];
        room.ground().reserve(director.players());
        let mut now = 0.0;
        while !director.finished() && now < 60.0 {
            now += 1.0 / 30.0;
            room.tick(&cast, now);
            director.run(room.ground(), &cast, now);
        }
        let (x, _) = room.ground().head(host, now).unwrap();
        let left = room.ground().spot("left", 0).0;
        assert!(
            (x - left).abs() < 12.0,
            "the walk to {left} was cut short at {x}"
        );
    }
}
