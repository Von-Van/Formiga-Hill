//! A story as the Clubhouse's story builder holds it while someone writes it: plain words in each
//! step, roles cast from a short list of plain choices, and scenes in the order they are played.
//! Nothing here is new to stories: a draft becomes an ordinary package, in exactly the format
//! `docs/PACKAGES.md` describes, and is loaded by the same loader as any other before it is
//! tried, kept on the board or shared.
//!
//! A draft says less than a story file can, on purpose. Each role is cast by one choice, with
//! "anyone" behind it unless it is left out when nobody fits; scenes follow one another unless
//! one says otherwise, and a choice for the person watching is the only way a story forks. A
//! line can still be said differently by each temperament, which is what makes the same story
//! play out differently in each colony.

use super::lines::KINDS;
use super::package::{Files, Package, load};
use super::script::{MAX_CHOICES, MAX_ROLES, MAX_SCENES, PLACES};
use super::{lines, souvenirs};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use toml::{Table, Value};

/// The most roles, scenes and steps the builder offers: room for a short story, well inside
/// what a package may hold.
pub const MOST_ROLES: usize = 8;
pub const MOST_SCENES: usize = 24;
pub const MOST_STEPS: usize = 60;
pub const MAX_TITLE_CHARS: usize = 60;
pub const MAX_NAME_CHARS: usize = 24;
pub const MAX_LINE_CHARS: usize = lines::MAX_LINE_CHARS;

/// Where a made story's package ids begin, and how the story inside is known.
const MADE_PREFIX: &str = "made.hill.";
const STORY_ID: &str = "story";
const STORY_FILE: &str = "content/story.toml";
const LINES_FILE: &str = "localization/en.toml";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Draft {
    pub title: String,
    pub author: String,
    /// The package id, made once when the draft is begun, so every version shared replaces
    /// the last wherever it went.
    pub package_id: String,
    /// Counts each time the story is put on the board or shared.
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub roles: Vec<DraftRole>,
    #[serde(default)]
    pub scenes: Vec<DraftScene>,
    /// The next number for a new scene; scenes keep theirs, so moving one never breaks a link.
    #[serde(default)]
    pub next_scene: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DraftRole {
    /// Lowercase letters, digits and `_`: what a line writes in braces to name whoever plays it.
    pub name: String,
    /// How the role is cast: one selector from `PICKS`, or one about an earlier role.
    pub pick: String,
    /// Left out if nobody fits, rather than played by anyone.
    #[serde(default)]
    pub optional: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DraftScene {
    pub number: u32,
    pub name: String,
    #[serde(default)]
    pub steps: Vec<Step>,
    /// Where the story goes once the scene's steps are done, unless it ends on a choice.
    #[serde(default)]
    pub then: Then,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Then {
    /// On to the scene after this one, or the end after the last.
    #[default]
    Next,
    Scene(u32),
    End,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub action: StepAction,
    /// Starts at the same time as the step before.
    #[serde(default)]
    pub together: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "do", rename_all = "snake_case")]
pub enum StepAction {
    Narrate {
        text: String,
    },
    Say {
        who: String,
        text: String,
        /// How a speaker of a temperament says it instead, by temperament.
        #[serde(default)]
        variants: BTreeMap<String, String>,
    },
    Walk {
        who: String,
        /// A place in the Clubhouse, or `beside:<role>`.
        to: String,
    },
    Face {
        who: String,
        to: String,
    },
    React {
        who: String,
        feeling: String,
    },
    Pose {
        who: String,
        pose: String,
    },
    Eat {
        who: String,
    },
    Nap {
        who: String,
    },
    Celebrate {
        who: String,
    },
    Play {
        who: String,
        with: String,
    },
    Wait {
        seconds: f32,
    },
    Choose {
        options: Vec<Choice>,
    },
    Souvenir {
        id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub text: String,
    /// The scene it leads to, by number.
    pub goto: u32,
}

/// Who a step can be about: a role, or everyone cast.
pub const EVERYONE: &str = "all";

/// The kinds of step the builder offers, in the order it lists them, and what each is called.
pub const KINDS_OF_STEP: [(&str, &str); 13] = [
    ("narrate", "The storyteller says something"),
    ("say", "Someone says something"),
    ("walk", "Someone goes somewhere"),
    ("face", "Someone turns to someone"),
    ("react", "Someone shows a feeling"),
    ("pose", "Someone does something"),
    ("eat", "Someone has a snack"),
    ("nap", "Someone takes a nap"),
    ("celebrate", "Someone celebrates"),
    ("play", "Two of them play together"),
    ("wait", "A pause"),
    ("choose", "A choice for whoever is watching"),
    ("souvenir", "A souvenir to keep"),
];

/// Ways to cast a role that need no other role, as the builder words them.
pub const PICKS: [(&str, &str); 27] = [
    ("any", "Anyone"),
    ("most:affection", "The most affectionate"),
    ("most:social", "The most sociable"),
    ("most:energy", "The most energetic"),
    ("most:boldness", "The boldest"),
    ("most:playfulness", "The most playful"),
    ("most:curiosity", "The most curious"),
    ("most:feistiness", "The feistiest"),
    ("most:impulsiveness", "The most impulsive"),
    ("most:suspicion", "The most suspicious"),
    ("least:boldness", "The shyest"),
    ("least:energy", "The sleepiest"),
    ("least:social", "The quietest"),
    ("least:suspicion", "The most trusting"),
    ("mini", "A little one"),
    ("adult", "A grown-up"),
    ("kind:sweetheart", "A sweetheart"),
    ("kind:troublemaker", "A troublemaker"),
    ("kind:grump", "A grump"),
    ("kind:explorer", "An explorer"),
    ("kind:wallflower", "A wallflower"),
    ("kind:showoff", "A show-off"),
    ("kind:scholar", "A scholar"),
    ("kind:oddball", "An oddball"),
    ("kind:lazybones", "A lazybones"),
    ("kind:guardian", "A guardian"),
    ("random", "Anyone, picked at random"),
];

/// Ways to cast a role by another, earlier role, and how each is worded with its name.
pub const RELATIONS: [(&str, &str); 5] = [
    ("friend_of", "{}'s closest friend"),
    ("playmate_of", "Someone {} plays with"),
    ("rival_of", "Someone {} doesn't get on with"),
    ("parent_of", "{}'s parent"),
    ("mini_of", "{}'s little one"),
];

/// The Clubhouse's places, as the builder words them, in `PLACES`' order.
pub const PLACE_NAMES: [(&str, &str); 13] = [
    ("rug", "the rug by the fire"),
    ("hearth", "the hearth"),
    ("armchair", "the big armchair"),
    ("bookshelf", "the bookshelf"),
    ("window", "the window"),
    ("board", "the notice board"),
    ("table", "the table"),
    ("chest", "the toy box"),
    ("centre", "the middle of the room"),
    ("left", "the left of the room"),
    ("right", "the right of the room"),
    ("front", "the front of the room"),
    ("back", "the back of the room"),
];

pub const FEELINGS: [(&str, &str); 10] = [
    ("joy", "joy"),
    ("surprise", "surprise"),
    ("worry", "worry"),
    ("fond", "fondness"),
    ("proud", "pride"),
    ("sleepy", "sleepiness"),
    ("grumpy", "grumpiness"),
    ("curious", "curiosity"),
    ("bored", "boredom"),
    ("shy", "shyness"),
];

pub const POSES: [(&str, &str); 14] = [
    ("inspect", "looks closely"),
    ("sit", "sits down"),
    ("crouch", "crouches"),
    ("stretch", "stretches"),
    ("wave", "waves"),
    ("peek", "peeks"),
    ("strut", "struts"),
    ("cheer", "cheers"),
    ("dance", "dances"),
    ("huff", "huffs"),
    ("beg", "begs"),
    ("watch", "watches"),
    ("cover", "covers their eyes"),
    ("balance", "balances"),
];

/// The temperaments a line can be said differently by, as the builder words them, in `KINDS`'
/// order.
pub const KIND_NAMES: [&str; 10] = [
    "A sweetheart",
    "A troublemaker",
    "A grump",
    "An explorer",
    "A wallflower",
    "A show-off",
    "A scholar",
    "An oddball",
    "A lazybones",
    "A guardian",
];

fn named<'a>(table: &'a [(&str, &str)], key: &str) -> Option<&'a str> {
    table
        .iter()
        .find(|(known, _)| *known == key)
        .map(|(_, name)| *name)
}

/// How a role's casting reads, in the builder's words.
pub fn pick_name(pick: &str) -> String {
    if let Some(name) = named(&PICKS, pick) {
        return name.to_owned();
    }
    if let Some((verb, role)) = pick.split_once(':')
        && let Some(words) = named(&RELATIONS, verb)
    {
        return words.replace("{}", role);
    }
    pick.to_owned()
}

pub fn place_name(place: &str) -> String {
    match place.strip_prefix("beside:") {
        Some(role) => format!("beside {role}"),
        None => named(&PLACE_NAMES, place).unwrap_or(place).to_owned(),
    }
}

pub fn feeling_name(feeling: &str) -> &str {
    named(&FEELINGS, feeling).unwrap_or(feeling)
}

pub fn pose_name(pose: &str) -> &str {
    named(&POSES, pose).unwrap_or(pose)
}

pub fn kind_name(kind: &str) -> &str {
    KINDS
        .iter()
        .position(|known| *known == kind)
        .map_or(kind, |index| KIND_NAMES[index])
}

pub fn who_name(who: &str) -> &str {
    if who == EVERYONE { "everyone" } else { who }
}

/// Where something wrong with a draft is, so the builder can show it there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spot {
    Story,
    Role(usize),
    Scene(usize),
    Step(usize, usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub spot: Spot,
    pub words: String,
}

impl Draft {
    /// A new story, begun with a little of one, so it is plain how one goes: someone fond of
    /// company and their closest friend, and a first scene by the fire.
    pub fn begin(key: &str) -> Self {
        let mut draft = Self {
            title: "A new story".into(),
            author: String::new(),
            package_id: format!("{MADE_PREFIX}{}", crate::sharing::slug(key, 24)),
            version: 0,
            roles: vec![
                DraftRole {
                    name: "host".into(),
                    pick: "most:affection".into(),
                    optional: false,
                },
                DraftRole {
                    name: "friend".into(),
                    pick: "friend_of:host".into(),
                    optional: false,
                },
            ],
            scenes: Vec::new(),
            next_scene: 1,
        };
        let number = draft.add_scene("The beginning");
        let scene = draft.scene_mut(number).expect("just added");
        scene.steps = vec![
            Step::new(StepAction::Narrate {
                text: "It is a quiet afternoon in the Clubhouse.".into(),
            }),
            Step::new(StepAction::Walk {
                who: "host".into(),
                to: "rug".into(),
            }),
            Step::new(StepAction::Walk {
                who: "friend".into(),
                to: "beside:host".into(),
            })
            .together(),
            Step::new(StepAction::Say {
                who: "host".into(),
                text: "Hello, {friend}! Come and sit by the fire.".into(),
                variants: BTreeMap::new(),
            }),
        ];
        draft
    }

    /// Adds a scene at the end, and says its number.
    pub fn add_scene(&mut self, name: &str) -> u32 {
        let number = self.next_scene.max(1);
        self.next_scene = number + 1;
        self.scenes.push(DraftScene {
            number,
            name: name.to_owned(),
            steps: Vec::new(),
            then: Then::Next,
        });
        number
    }

    pub fn scene_mut(&mut self, number: u32) -> Option<&mut DraftScene> {
        self.scenes.iter_mut().find(|scene| scene.number == number)
    }

    pub fn scene_index(&self, number: u32) -> Option<usize> {
        self.scenes.iter().position(|scene| scene.number == number)
    }

    /// A scene's name, for choosing it in a list.
    pub fn scene_name(&self, number: u32) -> String {
        self.scene_index(number).map_or_else(
            || "a scene that has gone".to_owned(),
            |index| scene_label(index, &self.scenes[index].name),
        )
    }

    /// Takes a scene out, and anything that went to it goes on to the next scene instead.
    pub fn remove_scene(&mut self, index: usize) {
        if index >= self.scenes.len() || self.scenes.len() == 1 {
            return;
        }
        let gone = self.scenes.remove(index).number;
        let fallback = self
            .scenes
            .get(index.min(self.scenes.len() - 1))
            .map(|s| s.number);
        for scene in &mut self.scenes {
            if scene.then == Then::Scene(gone) {
                scene.then = Then::Next;
            }
            for step in &mut scene.steps {
                if let StepAction::Choose { options } = &mut step.action {
                    for option in options {
                        if option.goto == gone
                            && let Some(fallback) = fallback
                        {
                            option.goto = fallback;
                        }
                    }
                }
            }
        }
    }

    /// Renames a role everywhere it is named: in casting, in steps, and in braces in lines.
    pub fn rename_role(&mut self, index: usize, new: &str) {
        let Some(role) = self.roles.get(index) else {
            return;
        };
        let old = role.name.clone();
        if old == new {
            return;
        }
        let swap = |name: &mut String| {
            if *name == old {
                *name = new.to_owned();
            }
        };
        let swap_place = |place: &mut String| {
            if place.strip_prefix("beside:") == Some(&old) {
                *place = format!("beside:{new}");
            }
        };
        let old_brace = format!("{{{old}}}");
        let new_brace = format!("{{{new}}}");
        let swap_text = |text: &mut String| *text = text.replace(&old_brace, &new_brace);
        self.roles[index].name = new.to_owned();
        for role in &mut self.roles {
            if let Some((verb, other)) = role.pick.split_once(':')
                && other == old
                && RELATIONS.iter().any(|(known, _)| *known == verb)
            {
                role.pick = format!("{verb}:{new}");
            }
        }
        for scene in &mut self.scenes {
            for step in &mut scene.steps {
                match &mut step.action {
                    StepAction::Narrate { text } => swap_text(text),
                    StepAction::Say {
                        who,
                        text,
                        variants,
                    } => {
                        swap(who);
                        swap_text(text);
                        variants.values_mut().for_each(swap_text);
                    }
                    StepAction::Walk { who, to } => {
                        swap(who);
                        swap_place(to);
                    }
                    StepAction::Face { who, to } | StepAction::Play { who, with: to } => {
                        swap(who);
                        swap(to);
                    }
                    StepAction::React { who, .. }
                    | StepAction::Pose { who, .. }
                    | StepAction::Eat { who }
                    | StepAction::Nap { who }
                    | StepAction::Celebrate { who } => swap(who),
                    StepAction::Choose { options } => {
                        options.iter_mut().for_each(|o| swap_text(&mut o.text));
                    }
                    StepAction::Wait { .. } | StepAction::Souvenir { .. } => {}
                }
            }
        }
    }

    /// A name for a new role that no role has yet.
    pub fn free_role_name(&self) -> String {
        for name in ["guest", "helper", "visitor", "neighbour", "pal", "chum"] {
            if !self.roles.iter().any(|role| role.name == name) {
                return name.to_owned();
            }
        }
        (2..)
            .map(|n| format!("guest{n}"))
            .find(|name| !self.roles.iter().any(|role| role.name == *name))
            .expect("there is always another number")
    }

    /// A step of `kind`, from `KINDS_OF_STEP`, filled in with something sensible to change.
    pub fn new_step(&self, kind: &str, scene: usize) -> Step {
        let first = self
            .roles
            .first()
            .map_or_else(|| EVERYONE.to_owned(), |role| role.name.clone());
        let second = self
            .roles
            .get(1)
            .map_or_else(|| first.clone(), |role| role.name.clone());
        let other_scene = self
            .scenes
            .get(scene + 1)
            .or(self.scenes.first())
            .map_or(0, |s| s.number);
        Step::new(match kind {
            "narrate" => StepAction::Narrate {
                text: String::new(),
            },
            "say" => StepAction::Say {
                who: first,
                text: String::new(),
                variants: BTreeMap::new(),
            },
            "walk" => StepAction::Walk {
                who: first,
                to: "centre".into(),
            },
            "face" => StepAction::Face {
                who: first,
                to: second,
            },
            "react" => StepAction::React {
                who: first,
                feeling: "joy".into(),
            },
            "pose" => StepAction::Pose {
                who: first,
                pose: "wave".into(),
            },
            "eat" => StepAction::Eat { who: first },
            "nap" => StepAction::Nap { who: first },
            "celebrate" => StepAction::Celebrate { who: first },
            "play" => StepAction::Play {
                who: first,
                with: second,
            },
            "wait" => StepAction::Wait { seconds: 2.0 },
            "choose" => StepAction::Choose {
                options: vec![
                    Choice {
                        text: String::new(),
                        goto: other_scene,
                    },
                    Choice {
                        text: String::new(),
                        goto: other_scene,
                    },
                ],
            },
            _ => StepAction::Souvenir {
                id: souvenirs::for_stories()[0].to_owned(),
            },
        })
    }

    /// The fewest travellers it needs: one for each role that must be cast.
    pub fn min_cast(&self) -> usize {
        self.roles
            .iter()
            .filter(|role| !role.optional)
            .count()
            .max(1)
    }

    /// Everything that would stop the story loading, said where it is, in the builder's words.
    pub fn problems(&self) -> Vec<Problem> {
        let mut problems = Vec::new();
        let mut say = |spot: Spot, words: String| problems.push(Problem { spot, words });
        if !is_line(&self.title, MAX_TITLE_CHARS) {
            say(
                Spot::Story,
                format!("Give the story a title, of up to {MAX_TITLE_CHARS} letters."),
            );
        }
        if !is_line(&self.author, MAX_TITLE_CHARS) {
            say(
                Spot::Story,
                "Say who wrote it: a name to show beside the title.".into(),
            );
        }
        if self.roles.is_empty() {
            say(
                Spot::Story,
                "A story needs someone in it: add a role.".into(),
            );
        }
        let names: Vec<&str> = self.roles.iter().map(|role| role.name.as_str()).collect();
        for (index, role) in self.roles.iter().enumerate() {
            if !is_role_name(&role.name) {
                say(
                    Spot::Role(index),
                    "A role's name is one short word of small letters, such as host.".into(),
                );
            } else if names[..index].contains(&role.name.as_str()) {
                say(
                    Spot::Role(index),
                    format!("Two roles are called {}.", role.name),
                );
            }
            match role.pick.split_once(':') {
                Some((verb, other)) if RELATIONS.iter().any(|(known, _)| *known == verb) => {
                    if !names[..index].contains(&other) {
                        say(
                            Spot::Role(index),
                            format!(
                                "{} is cast by {other}, so {other} needs to come above it in the cast.",
                                role.name
                            ),
                        );
                    }
                }
                _ if named(&PICKS, &role.pick).is_none() => {
                    say(Spot::Role(index), "Choose who plays this role.".into());
                }
                _ => {}
            }
        }
        let is_role = |name: &str| names.contains(&name);
        let numbers: Vec<u32> = self.scenes.iter().map(|scene| scene.number).collect();
        if self.scenes.is_empty() {
            say(Spot::Story, "A story needs a scene.".into());
        }
        for (s, scene) in self.scenes.iter().enumerate() {
            if !is_line(&scene.name, MAX_TITLE_CHARS) {
                say(Spot::Scene(s), "Give the scene a name.".into());
            }
            if let Then::Scene(to) = scene.then
                && !numbers.contains(&to)
            {
                say(
                    Spot::Scene(s),
                    "Say where the story goes after this scene.".into(),
                );
            }
            let last = scene.steps.len().saturating_sub(1);
            for (b, step) in scene.steps.iter().enumerate() {
                let spot = Spot::Step(s, b);
                let mut wrong = |words: String| say(spot, words);
                let check_who = |who: &str, everyone: bool, wrong: &mut dyn FnMut(String)| {
                    if !(is_role(who) || everyone && who == EVERYONE) {
                        wrong("Choose who this is about.".into());
                    }
                };
                let check_text = |text: &str, wrong: &mut dyn FnMut(String)| {
                    if let Some(words) = text_problem(text, &names) {
                        wrong(words);
                    }
                };
                match &step.action {
                    StepAction::Narrate { text } => check_text(text, &mut wrong),
                    StepAction::Say {
                        who,
                        text,
                        variants,
                    } => {
                        check_who(who, false, &mut wrong);
                        check_text(text, &mut wrong);
                        for (kind, words) in variants {
                            if !KINDS.contains(&kind.as_str()) {
                                wrong("A version of the line is for no temperament.".into());
                            }
                            if let Some(problem) = text_problem(words, &names) {
                                wrong(format!("{}'s version: {problem}", kind_name(kind)));
                            }
                        }
                    }
                    StepAction::Walk { who, to } => {
                        check_who(who, true, &mut wrong);
                        let fine = match to.strip_prefix("beside:") {
                            Some(other) => is_role(other) && other != who,
                            None => PLACES.contains(&to.as_str()),
                        };
                        if !fine {
                            wrong("Choose where they go.".into());
                        }
                    }
                    StepAction::Face { who, to } => {
                        check_who(who, true, &mut wrong);
                        if !is_role(to) || who == to {
                            wrong("Choose who they turn to.".into());
                        }
                    }
                    StepAction::React { who, feeling } => {
                        check_who(who, true, &mut wrong);
                        if named(&FEELINGS, feeling).is_none() {
                            wrong("Choose a feeling.".into());
                        }
                    }
                    StepAction::Pose { who, pose } => {
                        check_who(who, true, &mut wrong);
                        if named(&POSES, pose).is_none() {
                            wrong("Choose what they do.".into());
                        }
                    }
                    StepAction::Eat { who }
                    | StepAction::Nap { who }
                    | StepAction::Celebrate { who } => check_who(who, true, &mut wrong),
                    StepAction::Play { who, with } => {
                        check_who(who, false, &mut wrong);
                        if !is_role(with) || who == with {
                            wrong("Choose who they play with.".into());
                        }
                    }
                    StepAction::Wait { seconds } => {
                        if !(0.0..=30.0).contains(seconds) {
                            wrong("A pause lasts up to 30 seconds.".into());
                        }
                    }
                    StepAction::Choose { options } => {
                        if options.is_empty() || options.len() > MAX_CHOICES {
                            wrong(format!("A choice has between 1 and {MAX_CHOICES} options."));
                        }
                        if b != last {
                            wrong("A choice ends its scene: move the steps after it into a scene of their own.".into());
                        }
                        for option in options {
                            if let Some(problem) = text_problem(&option.text, &names) {
                                wrong(format!("An option: {problem}"));
                            }
                            if !numbers.contains(&option.goto) {
                                wrong("Choose the scene each option leads to.".into());
                            }
                        }
                    }
                    StepAction::Souvenir { id } => {
                        if !souvenirs::a_story_can_give(id) {
                            wrong("Choose a souvenir.".into());
                        }
                    }
                }
            }
        }
        problems
    }

    /// The story as a package: a manifest, the story file and its words.
    pub fn files(&self) -> Files {
        let mut files = Files::new();
        files.insert("manifest.toml".into(), self.manifest().into_bytes());
        let (story, words) = self.story_and_lines();
        files.insert(STORY_FILE.into(), story.into_bytes());
        files.insert(LINES_FILE.into(), words.into_bytes());
        files
    }

    /// The story as a package, loaded by the same loader as every other: what is tried, put on
    /// the board and shared. Its problems in the builder's words come first; anything only the
    /// loader catches is said as plainly as it can be.
    pub fn package(&self) -> Result<Package, Vec<Problem>> {
        let problems = self.problems();
        if !problems.is_empty() {
            return Err(problems);
        }
        load(&self.files(), &self.title).map_err(|error| {
            vec![Problem {
                spot: Spot::Story,
                words: self.plainly(&error.problem),
            }]
        })
    }

    /// The loader's words about the story file, with its scene ids as the builder names them.
    fn plainly(&self, problem: &str) -> String {
        let mut words = problem.to_owned();
        for (index, scene) in self.scenes.iter().enumerate() {
            words = words.replace(
                &format!("scene \"s{}\"", scene.number),
                &format!("\u{201c}{}\u{201d}", scene_label(index, &scene.name)),
            );
        }
        words
    }

    fn manifest(&self) -> String {
        let mut manifest = Table::new();
        manifest.insert("package_id".into(), self.package_id.clone().into());
        manifest.insert("title".into(), self.title.trim().into());
        manifest.insert("author".into(), self.author.trim().into());
        manifest.insert("version".into(), self.version.max(1).to_string().into());
        manifest.insert("hill_api".into(), Value::Integer(2));
        manifest.insert("content_types".into(), vec![Value::from("story")].into());
        manifest.insert("entry_points".into(), vec![Value::from(STORY_FILE)].into());
        manifest.insert("default_locale".into(), "en".into());
        let mut requirements = Table::new();
        requirements.insert("min_cast".into(), Value::Integer(self.min_cast() as i64));
        requirements.insert("areas".into(), vec![Value::from("clubhouse")].into());
        manifest.insert("requirements".into(), requirements.into());
        format!(
            "# Written in the story builder at Formiga Hill.\n{}",
            toml::to_string(&manifest).unwrap_or_default()
        )
    }

    fn story_and_lines(&self) -> (String, String) {
        let mut words = Table::new();
        words.insert("title".into(), self.title.trim().into());
        let mut story = Table::new();
        let mut header = Table::new();
        header.insert("id".into(), STORY_ID.into());
        header.insert("title".into(), "title".into());
        header.insert("area".into(), "clubhouse".into());
        header.insert(
            "start".into(),
            self.scenes
                .first()
                .map_or_else(String::new, |scene| scene_id(scene.number))
                .into(),
        );
        story.insert("story".into(), header.into());

        let mut roles = Table::new();
        for (order, role) in self.roles.iter().enumerate() {
            let mut table = Table::new();
            table.insert("order".into(), Value::Integer(order as i64 + 1));
            let mut select = vec![Value::from(role.pick.clone())];
            if !role.optional && !matches!(role.pick.as_str(), "any" | "random") {
                select.push("any".into());
            }
            table.insert("select".into(), select.into());
            if role.optional {
                table.insert("optional".into(), true.into());
            }
            roles.insert(role.name.clone(), table.into());
        }
        story.insert("roles".into(), roles.into());

        let mut scenes = Vec::new();
        for (index, scene) in self.scenes.iter().enumerate() {
            let id = scene_id(scene.number);
            let mut said = Table::new();
            let mut beats = Vec::new();
            let mut line = |text: &str, variants: &BTreeMap<String, String>| -> String {
                let key = format!("l{}", said.len() + 1);
                let value: Value = if variants.is_empty() {
                    text.trim().into()
                } else {
                    let mut table = Table::new();
                    table.insert("text".into(), text.trim().into());
                    for (kind, words) in variants {
                        table.insert(kind.clone(), words.trim().into());
                    }
                    table.into()
                };
                said.insert(key.clone(), value);
                format!("{id}.{key}")
            };
            let none = BTreeMap::new();
            let mut ends_on_choice = false;
            for step in &scene.steps {
                let mut beat = Table::new();
                let mut put = |key: &str, value: Value| {
                    beat.insert(key.to_owned(), value);
                };
                match &step.action {
                    StepAction::Narrate { text } => put("narrate", line(text, &none).into()),
                    StepAction::Say {
                        who,
                        text,
                        variants,
                    } => {
                        put("say", who.clone().into());
                        put("line", line(text, variants).into());
                    }
                    StepAction::Walk { who, to } => {
                        put("walk", who.clone().into());
                        put("to", to.clone().into());
                    }
                    StepAction::Face { who, to } => {
                        put("face", who.clone().into());
                        put("to", to.clone().into());
                    }
                    StepAction::React { who, feeling } => {
                        put("react", who.clone().into());
                        put("feeling", feeling.clone().into());
                    }
                    StepAction::Pose { who, pose } => {
                        put("pose", who.clone().into());
                        put("as", pose.clone().into());
                    }
                    StepAction::Eat { who } => put("eat", who.clone().into()),
                    StepAction::Nap { who } => put("nap", who.clone().into()),
                    StepAction::Celebrate { who } => put("celebrate", who.clone().into()),
                    StepAction::Play { who, with } => {
                        put("play", who.clone().into());
                        put("with", with.clone().into());
                    }
                    StepAction::Wait { seconds } => {
                        put("wait", Value::Float(f64::from(*seconds)));
                    }
                    StepAction::Choose { options } => {
                        ends_on_choice = true;
                        let options: Vec<Value> = options
                            .iter()
                            .map(|option| {
                                let mut choice = Table::new();
                                choice.insert("line".into(), line(&option.text, &none).into());
                                choice.insert("goto".into(), scene_id(option.goto).into());
                                choice.into()
                            })
                            .collect();
                        put("choose", options.into());
                    }
                    StepAction::Souvenir { id } => put("souvenir", id.clone().into()),
                }
                if step.together {
                    beat.insert("meanwhile".into(), true.into());
                }
                beats.push(Value::from(beat));
            }
            if !ends_on_choice {
                let mut beat = Table::new();
                match scene.then {
                    Then::Next => match self.scenes.get(index + 1) {
                        Some(next) => {
                            beat.insert("goto".into(), scene_id(next.number).into());
                        }
                        None => {
                            beat.insert("end".into(), true.into());
                        }
                    },
                    Then::Scene(to) => {
                        beat.insert("goto".into(), scene_id(to).into());
                    }
                    Then::End => {
                        beat.insert("end".into(), true.into());
                    }
                }
                beats.push(beat.into());
            }
            let mut table = Table::new();
            table.insert("id".into(), id.clone().into());
            table.insert("beats".into(), beats.into());
            scenes.push(Value::from(table));
            if !said.is_empty() {
                words.insert(id, said.into());
            }
        }
        story.insert("scenes".into(), scenes.into());
        (
            format!(
                "# \u{201c}{}\u{201d}, written in the story builder at Formiga Hill.\n{}",
                self.title.trim(),
                toml::to_string(&story).unwrap_or_default()
            ),
            format!(
                "# Every word of \u{201c}{}\u{201d}. {{role}} is the name of whoever plays it.\n{}",
                self.title.trim(),
                toml::to_string(&words).unwrap_or_default()
            ),
        )
    }

    /// The draft as text to keep.
    pub fn to_text(&self) -> String {
        toml::to_string(self).unwrap_or_default()
    }

    pub fn from_text(text: &str) -> Option<Self> {
        let mut draft: Self = toml::from_str(text).ok()?;
        // However it was edited by hand, it stays within what the builder offers.
        draft.roles.truncate(MOST_ROLES.min(MAX_ROLES));
        draft.scenes.truncate(MOST_SCENES.min(MAX_SCENES));
        for scene in &mut draft.scenes {
            scene.steps.truncate(MOST_STEPS);
        }
        let highest = draft
            .scenes
            .iter()
            .map(|scene| scene.number)
            .max()
            .unwrap_or(0);
        draft.next_scene = draft.next_scene.max(highest + 1);
        if !draft.package_id.starts_with(MADE_PREFIX) {
            return None;
        }
        Some(draft)
    }
}

impl Step {
    pub fn new(action: StepAction) -> Self {
        Self {
            action,
            together: false,
        }
    }

    fn together(mut self) -> Self {
        self.together = true;
        self
    }

    /// The kind of step it is, as `KINDS_OF_STEP` names it.
    pub fn kind(&self) -> &'static str {
        match self.action {
            StepAction::Narrate { .. } => "narrate",
            StepAction::Say { .. } => "say",
            StepAction::Walk { .. } => "walk",
            StepAction::Face { .. } => "face",
            StepAction::React { .. } => "react",
            StepAction::Pose { .. } => "pose",
            StepAction::Eat { .. } => "eat",
            StepAction::Nap { .. } => "nap",
            StepAction::Celebrate { .. } => "celebrate",
            StepAction::Play { .. } => "play",
            StepAction::Wait { .. } => "wait",
            StepAction::Choose { .. } => "choose",
            StepAction::Souvenir { .. } => "souvenir",
        }
    }

    /// The people it is about, in the order they come in it.
    pub fn who(&self) -> Option<&str> {
        match &self.action {
            StepAction::Say { who, .. }
            | StepAction::Walk { who, .. }
            | StepAction::Face { who, .. }
            | StepAction::React { who, .. }
            | StepAction::Pose { who, .. }
            | StepAction::Eat { who }
            | StepAction::Nap { who }
            | StepAction::Celebrate { who }
            | StepAction::Play { who, .. } => Some(who),
            _ => None,
        }
    }
}

fn scene_id(number: u32) -> String {
    format!("s{number}")
}

/// A scene as the builder lists it: its place in the story and its name.
pub fn scene_label(index: usize, name: &str) -> String {
    format!("{}. {}", index + 1, name.trim())
}

fn is_line(text: &str, most: usize) -> bool {
    super::package::is_label(text, most)
}

pub fn is_role_name(name: &str) -> bool {
    (1..=MAX_NAME_CHARS).contains(&name.len())
        && name.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && !matches!(name, "all" | "you")
}

/// A role's name as someone types it: small letters, digits and `_`, spaces becoming `_`.
pub fn tidy_role_name(typed: &str) -> String {
    typed
        .chars()
        .filter_map(|c| match c {
            ' ' | '-' => Some('_'),
            c if c.is_ascii_alphanumeric() || c == '_' => Some(c.to_ascii_lowercase()),
            _ => None,
        })
        .take(MAX_NAME_CHARS)
        .collect()
}

/// What is wrong with a line of the story, if anything: it says something, fits, stays on one
/// line, and names in braces only the story's roles.
fn text_problem(text: &str, roles: &[&str]) -> Option<String> {
    if text.trim().is_empty() {
        return Some("Write what is said.".into());
    }
    if text.chars().count() > MAX_LINE_CHARS {
        return Some(format!("Keep it to {MAX_LINE_CHARS} letters."));
    }
    if text.chars().any(char::is_control) {
        return Some("Keep it to one line.".into());
    }
    let mut rest = text;
    while let Some(open) = rest.find(['{', '}']) {
        if rest[open..].starts_with('}') {
            return Some("A } has no { before it.".into());
        }
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            return Some("A { has no } after it.".into());
        };
        let name = &after[..close];
        if !roles.contains(&name) {
            return Some(format!(
                "{{{name}}} isn't a role in this story: a name in braces is a role, such as {{{}}}.",
                roles.first().copied().unwrap_or("host")
            ));
        }
        rest = &after[close + 1..];
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::story::Director;

    fn written() -> Draft {
        let mut draft = Draft::begin("test");
        draft.title = "The Lost Kite".into();
        draft.author = "Jake".into();
        draft.roles.push(DraftRole {
            name: "shy".into(),
            pick: "least:boldness".into(),
            optional: true,
        });
        let fetch = draft.add_scene("Fetching it");
        let leave = draft.add_scene("Leaving it");
        let end = draft.add_scene("Home");
        let first = draft.scenes[0].number;
        let scene = draft.scene_mut(first).unwrap();
        scene.steps.push(Step::new(StepAction::Say {
            who: "friend".into(),
            text: "A kite, {host}, stuck on the bookshelf!".into(),
            variants: [("grump".to_owned(), "Hmph. A kite.".to_owned())].into(),
        }));
        scene.steps.push(Step::new(StepAction::Choose {
            options: vec![
                Choice {
                    text: "Climb up for it".into(),
                    goto: fetch,
                },
                Choice {
                    text: "Leave it be".into(),
                    goto: leave,
                },
            ],
        }));
        let scene = draft.scene_mut(fetch).unwrap();
        scene.steps = vec![
            Step::new(StepAction::Walk {
                who: "host".into(),
                to: "bookshelf".into(),
            }),
            Step::new(StepAction::Pose {
                who: "shy".into(),
                pose: "peek".into(),
            })
            .together(),
            Step::new(StepAction::Celebrate {
                who: EVERYONE.into(),
            }),
            Step::new(StepAction::Souvenir {
                id: "swing_feather".into(),
            }),
        ];
        scene.then = Then::Scene(end);
        let scene = draft.scene_mut(leave).unwrap();
        scene.steps = vec![Step::new(StepAction::React {
            who: "friend".into(),
            feeling: "bored".into(),
        })];
        let scene = draft.scene_mut(end).unwrap();
        scene.steps = vec![Step::new(StepAction::Narrate {
            text: "And everyone went home for tea.".into(),
        })];
        draft
    }

    #[test]
    fn a_new_story_is_one_that_already_loads() {
        let mut draft = Draft::begin("abc");
        draft.author = "Someone".into();
        let package = draft.package().unwrap();
        assert_eq!(package.id, "made.hill.abc");
        assert_eq!(package.stories[0].min_cast, 2);
    }

    #[test]
    fn a_written_story_loads_as_any_package_does_and_plays_to_the_end() {
        let draft = written();
        let package = draft.package().unwrap();
        let story = &package.stories[0];
        assert_eq!(story.title, "The Lost Kite");
        assert_eq!(story.scenes.len(), 4);
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        for option in 0..2 {
            let mut room =
                crate::clubhouse::Clubhouse::open(&cast, 0.0, &Default::default(), Vec::new());
            let mut director = Director::new(story.clone(), &cast, 7).unwrap();
            room.ground().reserve(director.players());
            let mut now = 0.0;
            let mut said = Vec::new();
            while !director.finished() && now < 300.0 {
                now += 1.0 / 30.0;
                room.tick(&cast, now);
                director.run(room.ground(), &cast, now);
                if let Some(shown) = director.shown() {
                    said.push(shown.text.clone());
                    director.read_on();
                }
                if !director.choices().is_empty() {
                    director.choose(option);
                }
            }
            assert!(director.finished());
            assert!(said.iter().all(|line| !line.contains('{')), "{said:?}");
            assert_eq!(
                said.last().map(String::as_str),
                Some("And everyone went home for tea.")
            );
            assert_eq!(director.souvenirs().is_empty(), option == 1);
        }
    }

    #[test]
    fn a_draft_is_kept_and_read_back_unchanged() {
        let draft = written();
        assert_eq!(Draft::from_text(&draft.to_text()), Some(draft));
        assert_eq!(Draft::from_text("title = 3"), None);
    }

    #[test]
    fn what_is_wrong_is_said_where_it_is() {
        let mut draft = written();
        draft.author.clear();
        draft.roles[1].pick = "friend_of:shy".into();
        let first = draft.scenes[0].number;
        draft.scene_mut(first).unwrap().steps[0].action = StepAction::Narrate {
            text: "Hello {nobody}".into(),
        };
        let problems = draft.problems();
        let spots: Vec<Spot> = problems.iter().map(|p| p.spot).collect();
        assert!(spots.contains(&Spot::Story), "{problems:?}");
        assert!(spots.contains(&Spot::Role(1)), "{problems:?}");
        assert!(spots.contains(&Spot::Step(0, 0)), "{problems:?}");
        assert!(draft.package().is_err());
        // A choice partway through a scene would strand what comes after it.
        let mut draft = written();
        let first = draft.scenes[0].number;
        let scene = draft.scene_mut(first).unwrap();
        let choice = scene.steps.pop().unwrap();
        scene.steps.insert(0, choice);
        assert!(
            draft
                .problems()
                .iter()
                .any(|p| p.words.contains("ends its scene"))
        );
    }

    #[test]
    fn a_renamed_role_is_renamed_everywhere() {
        let mut draft = written();
        draft.rename_role(0, "baker");
        assert_eq!(draft.roles[1].pick, "friend_of:baker");
        let text = draft.to_text();
        assert!(!text.contains("host"), "{text}");
        assert!(text.contains("{baker}"));
        assert!(draft.package().is_ok(), "{:?}", draft.problems());
    }

    #[test]
    fn a_removed_scene_leaves_nothing_pointing_at_it() {
        let mut draft = written();
        draft.remove_scene(1);
        assert!(draft.problems().is_empty(), "{:?}", draft.problems());
        assert!(draft.package().is_ok());
    }

    #[test]
    fn every_kind_of_step_can_be_added_and_loads() {
        let mut draft = Draft::begin("kinds");
        draft.author = "Someone".into();
        draft.add_scene("Two");
        for (kind, _) in KINDS_OF_STEP {
            let mut step = draft.new_step(kind, 0);
            match &mut step.action {
                StepAction::Narrate { text } | StepAction::Say { text, .. } => {
                    *text = "Words.".into();
                }
                StepAction::Choose { options } => {
                    for option in options {
                        option.text = "This way".into();
                    }
                }
                _ => {}
            }
            let scene = &mut draft.scenes[0];
            scene.steps.retain(|step| step.kind() != "choose");
            scene.steps.push(step);
            assert!(draft.package().is_ok(), "{kind}: {:?}", draft.problems());
        }
    }

    #[test]
    fn every_way_of_casting_and_every_name_is_one_the_loader_knows() {
        for (pick, _) in PICKS {
            let mut draft = Draft::begin("picks");
            draft.author = "Someone".into();
            draft.roles[0].pick = pick.into();
            assert!(
                draft.package().is_ok(),
                "{pick}: {:?}",
                draft.package().err()
            );
        }
        for (relation, _) in RELATIONS {
            let mut draft = Draft::begin("relations");
            draft.author = "Someone".into();
            draft.roles[1].pick = format!("{relation}:host");
            assert!(draft.package().is_ok(), "{relation}");
        }
        let places: Vec<&str> = PLACE_NAMES.iter().map(|(place, _)| *place).collect();
        assert_eq!(places, PLACES);
        assert_eq!(KIND_NAMES.len(), KINDS.len());
    }

    #[test]
    fn a_role_name_is_tidied_as_it_is_typed() {
        assert_eq!(tidy_role_name("Big Sister!"), "big_sister");
        assert!(is_role_name("big_sister"));
        assert!(!is_role_name("all"));
        assert!(!is_role_name("2nd"));
    }
}
