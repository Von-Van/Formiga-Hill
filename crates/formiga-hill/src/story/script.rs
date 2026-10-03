//! A story as an author writes it: who is in it, where it happens, and its scenes, each a list
//! of beats. Everything is checked when the package loads, so a story that loads is one the
//! director can play from start to end without meeting anything it does not understand.
//!
//! ```toml
//! [story]
//! id = "first-picnic"
//! title = "title"            # a line in the localisation file
//! area = "clubhouse"
//! start = "spread"
//!
//! [roles.host]
//! select = ["most:affection", "any"]
//!
//! [[scenes]]
//! id = "spread"
//!
//! [[scenes.beats]]
//! walk = "host"
//! to = "rug"
//! ```

use super::lines::{Lines, fill};
use formiga_core::{Habit, TemperamentKind};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The official areas a story can be staged in. Stories moved indoors when the Clubhouse was
/// built, in content API 2, and the green went back to free play.
pub const AREAS: [&str; 1] = ["clubhouse"];
/// The named spots of the Clubhouse a beat can send someone to.
pub const PLACES: [&str; 13] = [
    "rug",
    "hearth",
    "armchair",
    "bookshelf",
    "window",
    "board",
    "table",
    "chest",
    "centre",
    "left",
    "right",
    "front",
    "back",
];
/// Content API 1 staged stories on the green. A story written for it still loads, and is played
/// in the Clubhouse: its area is read as the Clubhouse, and each of the green's places as its
/// counterpart there.
pub const API_1_AREA: &str = "green";
pub const API_1_PLACES: [(&str, &str); 10] = [
    // Where everyone gathers, sits and eats.
    ("blanket", "rug"),
    // The centrepiece at the back.
    ("well", "hearth"),
    // The tall thing at the back on the left, and somewhere to clamber beside it.
    ("oak", "bookshelf"),
    ("swing", "armchair"),
    ("chest", "chest"),
    ("centre", "centre"),
    ("left", "left"),
    ("right", "right"),
    ("front", "front"),
    ("back", "back"),
];
pub const MAX_ROLES: usize = 12;
pub const MAX_SCENES: usize = 64;
pub const MAX_BEATS: usize = 200;
pub const MAX_CHOICES: usize = 4;
pub const MAX_FLAGS: usize = 32;
const MAX_ID_CHARS: usize = 40;

#[derive(Clone, Debug)]
pub struct Story {
    pub id: String,
    /// The title, in the package's own words.
    pub title: String,
    /// Where it is staged, whatever the content API it was written for called it.
    pub area: &'static str,
    /// The content API it was written for.
    pub api: u32,
    /// For a story written for content API 1, each of the green's places it names that the
    /// Clubhouse calls something else, and what it is read as, for telling its author.
    pub read_as: Vec<(&'static str, &'static str)>,
    pub roles: Vec<Role>,
    pub scenes: Vec<Scene>,
    pub start: usize,
    pub lines: Lines,
    /// The fewest travellers it can be staged with: every role that must be cast, and at least
    /// what the package's manifest asks for.
    pub min_cast: usize,
}

#[derive(Clone, Debug)]
pub struct Role {
    pub name: String,
    /// Tried in order; the first that finds someone casts the role.
    pub select: Vec<Selector>,
    /// The story goes on without this role if nobody fits; its beats are skipped.
    pub optional: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Social,
    Energy,
    Boldness,
    Playfulness,
    Curiosity,
    Feistiness,
    Impulsiveness,
    Suspicion,
    Affection,
}

impl Axis {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "social" => Self::Social,
            "energy" => Self::Energy,
            "boldness" => Self::Boldness,
            "playfulness" => Self::Playfulness,
            "curiosity" => Self::Curiosity,
            "feistiness" => Self::Feistiness,
            "impulsiveness" => Self::Impulsiveness,
            "suspicion" => Self::Suspicion,
            "affection" => Self::Affection,
            _ => return None,
        })
    }

    pub fn of(self, axes: &formiga_core::Axes) -> f32 {
        match self {
            Self::Social => axes.social,
            Self::Energy => axes.energy,
            Self::Boldness => axes.boldness,
            Self::Playfulness => axes.playfulness,
            Self::Curiosity => axes.curiosity,
            Self::Feistiness => axes.feistiness,
            Self::Impulsiveness => axes.impulsiveness,
            Self::Suspicion => axes.suspicion,
            Self::Affection => axes.affection,
        }
    }
}

/// How a role finds someone to play it. Roles named here must be declared before this one.
#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Any,
    Random,
    Most(Axis),
    Least(Axis),
    Kind(TemperamentKind),
    /// One of the traits Desktop's notebook names, such as "Brave".
    Trait(String),
    Habit(Habit),
    FriendOf(usize),
    PlaymateOf(usize),
    RivalOf(usize),
    ParentOf(usize),
    MiniOf(usize),
    Mini,
    Adult,
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub beats: Vec<Beat>,
}

#[derive(Clone, Debug)]
pub struct Beat {
    pub action: Action,
    /// Starts alongside the beat before it rather than after it.
    pub meanwhile: bool,
    /// Played only if this holds; skipped otherwise.
    pub when: Option<Condition>,
}

/// Who a beat is about: one role, or everyone cast.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who {
    Role(usize),
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Named(&'static str),
    Beside(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feeling {
    Joy,
    Surprise,
    Worry,
    Fond,
    Proud,
    Sleepy,
    Grumpy,
    Curious,
    Bored,
    Shy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    Inspect,
    Sit,
    Crouch,
    Stretch,
    Wave,
    Peek,
    Strut,
    Cheer,
    Dance,
    Huff,
    Beg,
    Watch,
    Cover,
    Balance,
}

#[derive(Clone, Debug)]
pub enum Action {
    Walk { who: Who, to: Place },
    Face { who: Who, to: usize },
    React { who: Who, feeling: Feeling },
    Pose { who: Who, pose: Pose },
    Eat { who: Who },
    Nap { who: Who },
    Celebrate { who: Who },
    Play { who: usize, with: usize },
    Say { who: usize, line: String },
    Narrate { line: String, about: Option<usize> },
    Wait { seconds: f32 },
    Choose { options: Vec<(String, usize)> },
    Goto { scene: usize },
    If { condition: Condition, goto: usize },
    Set { flag: String },
    Souvenir { id: String },
    End,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    Flag(String),
    Present(usize),
    Mini(usize),
    Kind(usize, TemperamentKind),
    Trait(usize, String),
    Habit(usize, Habit),
    High(usize, Axis),
    Low(usize, Axis),
    Close(usize, usize),
    Rivals(usize, usize),
    Not(Box<Condition>),
}

// ---------------------------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------------------------

#[derive(Deserialize)]
struct RawStory {
    story: RawHeader,
    #[serde(default)]
    roles: BTreeMap<String, RawRole>,
    #[serde(default)]
    scenes: Vec<RawScene>,
}

#[derive(Deserialize)]
struct RawHeader {
    id: String,
    title: String,
    area: String,
    start: String,
}

#[derive(Deserialize)]
struct RawRole {
    select: Vec<String>,
    #[serde(default)]
    optional: bool,
    /// Where it comes in the casting order; roles are otherwise cast in name order.
    #[serde(default)]
    order: Option<u32>,
}

#[derive(Deserialize)]
struct RawScene {
    id: String,
    #[serde(default)]
    beats: Vec<RawBeat>,
}

#[derive(Default, Deserialize)]
struct RawBeat {
    walk: Option<String>,
    to: Option<String>,
    face: Option<String>,
    react: Option<String>,
    feeling: Option<String>,
    pose: Option<String>,
    #[serde(rename = "as")]
    pose_as: Option<String>,
    eat: Option<String>,
    nap: Option<String>,
    celebrate: Option<String>,
    play: Option<String>,
    with: Option<String>,
    say: Option<String>,
    line: Option<String>,
    narrate: Option<String>,
    about: Option<String>,
    wait: Option<f32>,
    choose: Option<Vec<RawChoice>>,
    goto: Option<String>,
    #[serde(rename = "if")]
    condition: Option<String>,
    set: Option<String>,
    souvenir: Option<String>,
    end: Option<bool>,
    #[serde(default)]
    meanwhile: bool,
    when: Option<String>,
}

#[derive(Deserialize)]
struct RawChoice {
    line: String,
    goto: String,
}

/// Parses and checks a story file against the package's lines. Any problem is reported with
/// where it is, in the author's terms.
/// Parses and checks a story file written for content API `api` against the package's lines.
pub fn parse_story(
    text: &str,
    lines: &Lines,
    package_min_cast: usize,
    api: u32,
) -> Result<Story, String> {
    let raw: RawStory = toml::from_str(text).map_err(|error| error.to_string())?;
    let header = raw.story;
    if !is_id(&header.id) {
        return Err("story.id must be short, lowercase, letters, digits and '-'".into());
    }
    let area =
        staged_area(api, &header.area).map_err(|problem| format!("story.area: {problem}"))?;

    // Roles, in casting order.
    if raw.roles.is_empty() || raw.roles.len() > MAX_ROLES {
        return Err(format!("a story casts between 1 and {MAX_ROLES} roles"));
    }
    let mut ordered: Vec<(&String, &RawRole)> = raw.roles.iter().collect();
    ordered.sort_by_key(|(name, role)| (role.order.unwrap_or(u32::MAX), (*name).clone()));
    let names: Vec<String> = ordered.iter().map(|(name, _)| (*name).clone()).collect();
    for name in &names {
        if !is_id(name) || name.contains('-') || matches!(name.as_str(), "all" | "you") {
            return Err(format!(
                "role \"{name}\" needs a plain name of lowercase letters, digits and '_'"
            ));
        }
    }
    let mut roles = Vec::new();
    for (index, (name, role)) in ordered.iter().enumerate() {
        let earlier = &names[..index];
        let select = role
            .select
            .iter()
            .map(|text| {
                parse_selector(text, earlier).map_err(|problem| format!("roles.{name}: {problem}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if select.is_empty() {
            return Err(format!("roles.{name} has nothing to select by"));
        }
        if !role.optional && !matches!(select.last(), Some(Selector::Any | Selector::Random)) {
            return Err(format!(
                "roles.{name} must end its select with \"any\" or \"random\", or be optional: a story cannot assume what a colony is like"
            ));
        }
        roles.push(Role {
            name: (*name).clone(),
            select,
            optional: role.optional,
        });
    }
    let role = |name: &str| names.iter().position(|known| known == name);

    // Scenes.
    if raw.scenes.is_empty() || raw.scenes.len() > MAX_SCENES {
        return Err(format!("a story has between 1 and {MAX_SCENES} scenes"));
    }
    let scene_ids: Vec<&str> = raw.scenes.iter().map(|scene| scene.id.as_str()).collect();
    for (index, id) in scene_ids.iter().enumerate() {
        if !is_id(id) || scene_ids[..index].contains(id) {
            return Err(format!("scene \"{id}\" needs a short, unique id"));
        }
    }
    let scene = |id: &str| {
        scene_ids
            .iter()
            .position(|known| *known == id)
            .ok_or_else(|| format!("there is no scene \"{id}\""))
    };
    let start = scene(&header.start).map_err(|problem| format!("story.start: {problem}"))?;

    let title = lines.get(&header.title).ok_or_else(|| {
        format!(
            "story.title \"{}\" is not in the localisation file",
            header.title
        )
    })?;
    let title = fill(&title.text, |_| None).ok_or("story.title cannot name a role")?;

    let check_line = |key: &str| -> Result<String, String> {
        let line = lines
            .get(key)
            .ok_or_else(|| format!("line \"{key}\" is not in the localisation file"))?;
        if let Some(stranger) = line
            .placeholders()
            .into_iter()
            .find(|name| role(name).is_none())
        {
            return Err(format!(
                "line \"{key}\" names {{{stranger}}}, which is not a role in this story"
            ));
        }
        Ok(key.to_owned())
    };

    let mut read_as = Vec::new();
    let mut place = |written: &str| -> Result<&'static str, String> {
        let (written, staged) = staged_place(api, written)?;
        if written != staged && !read_as.contains(&(written, staged)) {
            read_as.push((written, staged));
        }
        Ok(staged)
    };
    let mut flags = std::collections::BTreeSet::new();
    let mut scenes = Vec::new();
    for raw_scene in &raw.scenes {
        if raw_scene.beats.len() > MAX_BEATS {
            return Err(format!(
                "scene \"{}\" has more than {MAX_BEATS} beats",
                raw_scene.id
            ));
        }
        let mut beats = Vec::new();
        for (number, raw_beat) in raw_scene.beats.iter().enumerate() {
            let at = |problem: String| {
                format!("scene \"{}\", beat {}: {problem}", raw_scene.id, number + 1)
            };
            let beat = parse_beat(raw_beat, &role, &scene, &check_line, &mut place, &mut flags)
                .map_err(at)?;
            beats.push(beat);
        }
        scenes.push(Scene { beats });
    }
    if flags.len() > MAX_FLAGS {
        return Err(format!("a story keeps at most {MAX_FLAGS} flags"));
    }
    let required = roles.iter().filter(|role| !role.optional).count();
    Ok(Story {
        id: header.id,
        title,
        area,
        api,
        read_as,
        roles,
        scenes,
        start,
        lines: lines.clone(),
        min_cast: required.max(package_min_cast),
    })
}

fn parse_beat(
    raw: &RawBeat,
    role: &dyn Fn(&str) -> Option<usize>,
    scene: &dyn Fn(&str) -> Result<usize, String>,
    check_line: &dyn Fn(&str) -> Result<String, String>,
    place: &mut dyn FnMut(&str) -> Result<&'static str, String>,
    flags: &mut std::collections::BTreeSet<String>,
) -> Result<Beat, String> {
    let named = |text: &str| role(text).ok_or_else(|| format!("there is no role \"{text}\""));
    let who = |text: &str| -> Result<Who, String> {
        if text == "all" {
            Ok(Who::All)
        } else {
            named(text).map(Who::Role)
        }
    };
    let verbs = [
        raw.walk.is_some(),
        raw.face.is_some(),
        raw.react.is_some(),
        raw.pose.is_some(),
        raw.eat.is_some(),
        raw.nap.is_some(),
        raw.celebrate.is_some(),
        raw.play.is_some(),
        raw.say.is_some(),
        raw.narrate.is_some(),
        raw.wait.is_some(),
        raw.choose.is_some(),
        raw.condition.is_some(),
        raw.set.is_some(),
        raw.souvenir.is_some(),
        raw.end.is_some(),
        raw.goto.is_some() && raw.condition.is_none() && raw.choose.is_none(),
    ];
    match verbs.iter().filter(|verb| **verb).count() {
        1 => {}
        0 => return Err("a beat needs one thing to do, such as walk, say or choose".into()),
        _ => return Err("a beat does one thing; split this into several beats".into()),
    }
    let need = |value: &Option<String>, field: &str| {
        value
            .clone()
            .ok_or_else(|| format!("this beat needs `{field}`"))
    };

    let action = if let Some(text) = &raw.walk {
        let to = need(&raw.to, "to")?;
        let place = if let Some(other) = to.strip_prefix("beside:") {
            Place::Beside(named(other)?)
        } else {
            Place::Named(place(&to)?)
        };
        Action::Walk {
            who: who(text)?,
            to: place,
        }
    } else if let Some(text) = &raw.face {
        Action::Face {
            who: who(text)?,
            to: named(&need(&raw.to, "to")?)?,
        }
    } else if let Some(text) = &raw.react {
        let feeling = need(&raw.feeling, "feeling")?;
        Action::React {
            who: who(text)?,
            feeling: parse_feeling(&feeling)?,
        }
    } else if let Some(text) = &raw.pose {
        let pose = need(&raw.pose_as, "as")?;
        Action::Pose {
            who: who(text)?,
            pose: parse_pose(&pose)?,
        }
    } else if let Some(text) = &raw.eat {
        Action::Eat { who: who(text)? }
    } else if let Some(text) = &raw.nap {
        Action::Nap { who: who(text)? }
    } else if let Some(text) = &raw.celebrate {
        Action::Celebrate { who: who(text)? }
    } else if let Some(text) = &raw.play {
        Action::Play {
            who: named(text)?,
            with: named(&need(&raw.with, "with")?)?,
        }
    } else if let Some(text) = &raw.say {
        Action::Say {
            who: named(text)?,
            line: check_line(&need(&raw.line, "line")?)?,
        }
    } else if let Some(key) = &raw.narrate {
        Action::Narrate {
            line: check_line(key)?,
            about: raw.about.as_deref().map(named).transpose()?,
        }
    } else if let Some(seconds) = raw.wait {
        if !(0.0..=30.0).contains(&seconds) {
            return Err("a wait is between 0 and 30 seconds".into());
        }
        Action::Wait { seconds }
    } else if let Some(options) = &raw.choose {
        if options.is_empty() || options.len() > MAX_CHOICES {
            return Err(format!(
                "a choice offers between 1 and {MAX_CHOICES} options"
            ));
        }
        Action::Choose {
            options: options
                .iter()
                .map(|option| Ok((check_line(&option.line)?, scene(&option.goto)?)))
                .collect::<Result<_, String>>()?,
        }
    } else if let Some(text) = &raw.condition {
        Action::If {
            condition: parse_condition(text, role)?,
            goto: scene(&need(&raw.goto, "goto")?)?,
        }
    } else if let Some(flag) = &raw.set {
        if !is_id(flag) {
            return Err(format!("flag \"{flag}\" needs a short plain name"));
        }
        flags.insert(flag.clone());
        Action::Set { flag: flag.clone() }
    } else if let Some(id) = &raw.souvenir {
        if !super::souvenirs::a_story_can_give(id) {
            return Err(format!(
                "\"{id}\" is not one of the souvenirs a story can give ({})",
                super::souvenirs::for_stories().join(", ")
            ));
        }
        Action::Souvenir { id: id.clone() }
    } else if let Some(end) = raw.end {
        // `end = false` reads as "don't end here", so it must not quietly end the story.
        if !end {
            return Err("an end beat is `end = true`; to carry on, leave the beat out".into());
        }
        Action::End
    } else {
        Action::Goto {
            scene: scene(raw.goto.as_deref().unwrap_or_default())?,
        }
    };
    let when = raw
        .when
        .as_deref()
        .map(|text| parse_condition(text, role))
        .transpose()?;
    Ok(Beat {
        action,
        meanwhile: raw.meanwhile,
        when,
    })
}

/// Where a story written for content API `api`, naming `area`, is staged.
pub fn staged_area(api: u32, area: &str) -> Result<&'static str, String> {
    if api == 1 {
        return if area == API_1_AREA {
            Ok("clubhouse")
        } else {
            Err(format!(
                "\"{area}\" is not an area content API 1 knows: it knows only \"{API_1_AREA}\""
            ))
        };
    }
    AREAS
        .iter()
        .copied()
        .find(|known| *known == area)
        .ok_or_else(|| format!("\"{area}\" is not one of {}", AREAS.join(", ")))
}

/// A place as a story written for content API `api` names it, and where in the Clubhouse that is.
fn staged_place(api: u32, place: &str) -> Result<(&'static str, &'static str), String> {
    if api == 1 {
        return API_1_PLACES
            .iter()
            .copied()
            .find(|(green, _)| *green == place)
            .ok_or_else(|| {
                let names: Vec<&str> = API_1_PLACES.iter().map(|(green, _)| *green).collect();
                format!(
                    "\"{place}\" is not a place on the green ({}), or beside:<role>",
                    names.join(", ")
                )
            });
    }
    PLACES
        .iter()
        .copied()
        .find(|known| *known == place)
        .map(|known| (known, known))
        .ok_or_else(|| {
            format!(
                "\"{place}\" is not a place in the clubhouse ({}), or beside:<role>",
                PLACES.join(", ")
            )
        })
}

fn parse_selector(text: &str, earlier: &[String]) -> Result<Selector, String> {
    let named = |name: &str| {
        earlier
            .iter()
            .position(|known| known == name)
            .ok_or_else(|| {
                format!(
                    "\"{name}\" must be a role declared before this one (or given a lower `order`)"
                )
            })
    };
    let (verb, argument) = text.split_once(':').unwrap_or((text, ""));
    Ok(match verb {
        "any" => Selector::Any,
        "random" => Selector::Random,
        "mini" => Selector::Mini,
        "adult" => Selector::Adult,
        "most" | "least" => {
            let axis = Axis::parse(argument)
                .ok_or_else(|| format!("\"{argument}\" is not a temperament scale"))?;
            if verb == "most" {
                Selector::Most(axis)
            } else {
                Selector::Least(axis)
            }
        }
        "kind" => Selector::Kind(parse_kind(argument)?),
        "trait" if !argument.trim().is_empty() => Selector::Trait(argument.trim().to_owned()),
        "habit" => Selector::Habit(parse_habit(argument)?),
        "friend_of" => Selector::FriendOf(named(argument)?),
        "playmate_of" => Selector::PlaymateOf(named(argument)?),
        "rival_of" => Selector::RivalOf(named(argument)?),
        "parent_of" => Selector::ParentOf(named(argument)?),
        "mini_of" => Selector::MiniOf(named(argument)?),
        _ => return Err(format!("\"{text}\" is not a way of choosing someone")),
    })
}

fn parse_condition(text: &str, role: &dyn Fn(&str) -> Option<usize>) -> Result<Condition, String> {
    // The `!`s are counted rather than read one at a time, so however many a package puts in front
    // of a condition, Hill cannot run out of stack loading it. Two cancel out.
    let plain = text.trim_start_matches('!');
    let condition = parse_plain_condition(plain, role)?;
    Ok(if (text.len() - plain.len()) % 2 == 1 {
        Condition::Not(Box::new(condition))
    } else {
        condition
    })
}

fn parse_plain_condition(
    text: &str,
    role: &dyn Fn(&str) -> Option<usize>,
) -> Result<Condition, String> {
    let named = |name: &str| role(name).ok_or_else(|| format!("there is no role \"{name}\""));
    let (verb, argument) = text.split_once(':').ok_or_else(|| {
        format!("\"{text}\" is not a condition, such as flag:shared or kind:host=grump")
    })?;
    let pair = |argument: &str| -> Result<(usize, usize), String> {
        let (a, b) = argument
            .split_once(',')
            .ok_or("this needs two roles: a,b")?;
        Ok((named(a)?, named(b)?))
    };
    let assignment = |argument: &str| -> Result<(usize, String), String> {
        let (who, value) = argument.split_once('=').ok_or("this needs role=value")?;
        Ok((named(who)?, value.to_owned()))
    };
    let scale = |argument: &str| -> Result<(usize, Axis), String> {
        let (who, axis) = argument.split_once('.').ok_or("this needs role.scale")?;
        Ok((
            named(who)?,
            Axis::parse(axis).ok_or_else(|| format!("\"{axis}\" is not a temperament scale"))?,
        ))
    };
    Ok(match verb {
        // Flags may be set later in the story than they are first read; a flag never set is false.
        "flag" if is_id(argument) => Condition::Flag(argument.to_owned()),
        "present" => Condition::Present(named(argument)?),
        "mini" => Condition::Mini(named(argument)?),
        "kind" => {
            let (who, kind) = assignment(argument)?;
            Condition::Kind(who, parse_kind(&kind)?)
        }
        "trait" => {
            let (who, label) = assignment(argument)?;
            Condition::Trait(who, label)
        }
        "habit" => {
            let (who, habit) = assignment(argument)?;
            Condition::Habit(who, parse_habit(&habit)?)
        }
        "high" => {
            let (who, axis) = scale(argument)?;
            Condition::High(who, axis)
        }
        "low" => {
            let (who, axis) = scale(argument)?;
            Condition::Low(who, axis)
        }
        "close" => {
            let (a, b) = pair(argument)?;
            Condition::Close(a, b)
        }
        "rivals" => {
            let (a, b) = pair(argument)?;
            Condition::Rivals(a, b)
        }
        _ => return Err(format!("\"{text}\" is not a condition this Hill knows")),
    })
}

fn parse_kind(name: &str) -> Result<TemperamentKind, String> {
    TemperamentKind::ALL
        .into_iter()
        .find(|kind| super::lines::kind_key(*kind) == name)
        .ok_or_else(|| {
            format!(
                "\"{name}\" is not a temperament ({})",
                super::lines::KINDS.join(", ")
            )
        })
}

fn parse_habit(name: &str) -> Result<Habit, String> {
    Ok(match name {
        "looks_food_over" => Habit::LooksFoodOver,
        "stretches_before_naps" => Habit::StretchesBeforeNaps,
        "circles_before_naps" => Habit::CirclesBeforeNaps,
        "waves_hello" => Habit::WavesHello,
        "play_bows" => Habit::PlayBows,
        _ => return Err(format!("\"{name}\" is not a habit")),
    })
}

fn parse_feeling(name: &str) -> Result<Feeling, String> {
    Ok(match name {
        "joy" => Feeling::Joy,
        "surprise" => Feeling::Surprise,
        "worry" => Feeling::Worry,
        "fond" => Feeling::Fond,
        "proud" => Feeling::Proud,
        "sleepy" => Feeling::Sleepy,
        "grumpy" => Feeling::Grumpy,
        "curious" => Feeling::Curious,
        "bored" => Feeling::Bored,
        "shy" => Feeling::Shy,
        _ => {
            return Err(format!(
                "\"{name}\" is not a feeling (joy, surprise, worry, fond, proud, sleepy, grumpy, curious, bored, shy)"
            ));
        }
    })
}

fn parse_pose(name: &str) -> Result<Pose, String> {
    Ok(match name {
        "inspect" => Pose::Inspect,
        "sit" => Pose::Sit,
        "crouch" => Pose::Crouch,
        "stretch" => Pose::Stretch,
        "wave" => Pose::Wave,
        "peek" => Pose::Peek,
        "strut" => Pose::Strut,
        "cheer" => Pose::Cheer,
        "dance" => Pose::Dance,
        "huff" => Pose::Huff,
        "beg" => Pose::Beg,
        "watch" => Pose::Watch,
        "cover" => Pose::Cover,
        "balance" => Pose::Balance,
        _ => return Err(format!("\"{name}\" is not a pose")),
    })
}

fn is_id(text: &str) -> bool {
    (1..=MAX_ID_CHARS).contains(&text.len())
        && text.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINES: &str = r#"
        title = "Test"
        hello = "Hello, {friend}."
        pick_a = "One way"
        pick_b = "The other"
        stray = "Hello, {nobody}."
    "#;

    fn story(body: &str) -> Result<Story, String> {
        let text = format!(
            r#"
            [story]
            id = "test"
            title = "title"
            area = "clubhouse"
            start = "one"

            [roles.host]
            order = 1
            select = ["most:affection", "any"]
            [roles.friend]
            order = 2
            select = ["friend_of:host", "any"]
            [roles.shy]
            order = 3
            select = ["least:boldness"]
            optional = true

            {body}
            "#
        );
        parse_story(&text, &Lines::parse(LINES).unwrap(), 1, 2)
    }

    #[test]
    fn a_story_with_every_kind_of_beat_parses() {
        let parsed = story(
            r#"
            [[scenes]]
            id = "one"
            beats = [
              { walk = "host", to = "rug" },
              { walk = "friend", to = "beside:host", meanwhile = true },
              { react = "all", feeling = "joy" },
              { pose = "shy", as = "peek", when = "present:shy" },
              { say = "host", line = "hello" },
              { if = "kind:host=grump", goto = "two" },
              { set = "picked" },
              { choose = [ { line = "pick_a", goto = "two" }, { line = "pick_b", goto = "two" } ] },
            ]
            [[scenes]]
            id = "two"
            beats = [
              { play = "host", with = "friend" },
              { celebrate = "all", when = "flag:picked" },
              { souvenir = "picnic_ribbon" },
              { end = true },
            ]
            "#,
        )
        .unwrap();
        assert_eq!(parsed.roles.len(), 3);
        assert_eq!(parsed.scenes[0].beats.len(), 8);
        assert_eq!(
            parsed.min_cast, 2,
            "two roles must be cast; the shy one is optional"
        );
        assert!(parsed.scenes[0].beats[1].meanwhile);
    }

    #[test]
    fn mistakes_are_caught_and_named() {
        let cases = [
            (r#"{ walk = "host", to = "the moon" }"#, "not a place"),
            (r#"{ say = "ghost", line = "hello" }"#, "no role \"ghost\""),
            (
                r#"{ say = "host", line = "missing" }"#,
                "not in the localisation",
            ),
            (r#"{ say = "host", line = "stray" }"#, "{nobody}"),
            (r#"{ react = "host", feeling = "smug" }"#, "not a feeling"),
            (r#"{ goto = "nowhere" }"#, "no scene"),
            (
                r#"{ walk = "host", say = "host", line = "hello" }"#,
                "does one thing",
            ),
            (r#"{ souvenir = "a_real_diamond" }"#, "souvenirs"),
            (r#"{ if = "mood:host", goto = "one" }"#, "condition"),
            (r#"{ end = false }"#, "end = true"),
        ];
        for (beat, expected) in cases {
            let error =
                story(&format!("[[scenes]]\nid = \"one\"\nbeats = [ {beat} ]")).unwrap_err();
            assert!(error.contains(expected), "{beat} gave \"{error}\"");
        }
    }

    #[test]
    fn any_number_of_negations_loads_without_running_out_of_stack() {
        let odd = "!".repeat(100_001);
        let even = "!".repeat(100_000);
        let parsed = story(&format!(
            r#"
            [[scenes]]
            id = "one"
            beats = [
              {{ celebrate = "all", when = "{odd}flag:picked" }},
              {{ celebrate = "all", when = "{even}flag:picked" }},
            ]
            "#
        ))
        .unwrap();
        let picked = Condition::Flag("picked".into());
        assert_eq!(
            parsed.scenes[0].beats[0].when,
            Some(Condition::Not(Box::new(picked.clone())))
        );
        assert_eq!(parsed.scenes[0].beats[1].when, Some(picked));
    }

    #[test]
    fn every_place_on_the_green_is_read_as_a_place_in_the_clubhouse() {
        for (green, clubhouse) in API_1_PLACES {
            assert!(
                PLACES.contains(&clubhouse),
                "{green} is read as {clubhouse}"
            );
        }
    }

    #[test]
    fn a_role_that_must_be_cast_needs_a_fallback() {
        let text = r#"
            [story]
            id = "t"
            title = "title"
            area = "clubhouse"
            start = "one"
            [roles.brave]
            select = ["trait:Brave"]
            [[scenes]]
            id = "one"
        "#;
        let error = parse_story(text, &Lines::parse(LINES).unwrap(), 1, 2).unwrap_err();
        assert!(error.contains("must end its select"));
    }
}
