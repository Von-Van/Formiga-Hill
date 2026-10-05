//! A rest on the fallen log: the party sits along it and shares the snacks, and then each takes
//! the rest its own way. A lazybones dozes off in the sun, a little one snuggles up to its
//! parent, close friends lean together, and playmates can't sit still. It is there for the
//! sake of it, at a little cost of the day: nothing is won by it and nothing is owed.
//!
//! It is shown as a picture pinned over the map (see `scenery::picnic`), the party in it.

use crate::actor::Step;
use crate::cast::{Cast, Id};
use crate::character::{Beat, Character, Cue, Offer};
use crate::daylight::Nightlights;
use crate::dice::Dice;
use crate::playground::{Layout, Patch, Playground};
use formiga_art::{Canvas, ExpressionKind};
use formiga_core::{ActionKind, Gesture, TemperamentKind};
use formiga_travel::Band;

/// How long the rest lasts, unless the person moves on sooner.
pub const REST_SECS: f32 = 11.0;

/// The picture of the picnic, pinned over the map: left, top, width, height.
pub const PICTURE: (i32, i32, i32, i32) = (86, 30, 212, 140);
/// Where the party sits along the log, in the picture, middle first.
pub const SEATS: [(f32, f32); 3] = [(196.0, 128.0), (166.0, 131.0), (226.0, 130.0)];
/// The floor of the clearing in the picture, in front of the log.
const FLOOR: Patch = (104.0, 120.0, 280.0, 160.0);

fn walkable(x: f32, y: f32) -> bool {
    let (left, top, right, bottom) = FLOOR;
    x >= left && x <= right && y >= top && y <= bottom
}

const PLACES: [(&str, (f32, f32)); 1] = [("centre", (192.0, 150.0))];

fn layout() -> Layout {
    Layout {
        walkable,
        ground: FLOOR,
        spots: &PLACES,
        seats: None,
        shade: None,
        entrance: (110.0, 150.0),
        entrance_step: (6.0, 2.0),
    }
}

/// The picnic's picture over `map`, with the party sat along the log.
pub fn open(cast: &Cast, party: &[Id], map: &Canvas, now: f32) -> Playground {
    let mut ground = Playground::with_members(
        cast,
        party,
        now,
        layout(),
        super::scenery::picnic(map),
        Canvas::new(map.width(), map.height()),
        Vec::new(),
    );
    // Paper and a picture, lit as a room is: by lamplight after dark.
    ground.set_nightlights(Nightlights {
        lamps: Canvas::new(1, 1),
        sky: Canvas::new(1, 1),
        indoors: true,
    });
    ground
}

/// Something that happened, for the person to be told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Out come the snacks, shared along the log.
    Shared,
    /// Someone has dozed off in the sun.
    Dozed { who: Id },
    /// Two lean together: a little one and its parent, or close friends.
    Snuggled { a: Id, b: Id },
    /// Two playmates can't sit still.
    Played { a: Id, b: Id },
    /// The rest is over.
    Done,
}

pub struct Picnic {
    since: f32,
    over: bool,
    events: Vec<Event>,
}

impl Picnic {
    /// The party sits along the log in `ground` and the snacks come out. `cast` says how each
    /// pair gets on.
    pub fn new(ground: &mut Playground, cast: &Cast, party: &[Id], seed: u64, now: f32) -> Self {
        let mut dice = Dice::new(seed);
        let characters: Vec<(Id, Character)> = party
            .iter()
            .filter_map(|id| ground.character(*id).map(|c| (*id, c.clone())))
            .collect();
        ground.reserve(party.to_vec());
        // A little one sits beside its parent.
        let mut order: Vec<Id> = party.to_vec();
        if let Some(little) = characters
            .iter()
            .find(|(_, c)| c.parent.is_some_and(|parent| party.contains(&parent)))
            && let Some(parent) = little.1.parent
        {
            order.retain(|id| *id != little.0 && *id != parent);
            order.insert(0, parent);
            order.insert(1, little.0);
        }
        let mut events = vec![Event::Shared];
        let lazy = |character: &Character| {
            character.kind == TemperamentKind::Lazybones || character.axes.energy < 0.2
        };
        // Who keeps whom company, each once: its parent or little one first, then a close
        // friend, then a playmate for a playful one. A lazybones is asleep before long.
        let mut partners: Vec<(Id, Id, bool)> = Vec::new();
        let taken = |partners: &[(Id, Id, bool)], id: Id| {
            partners.iter().any(|(a, b, _)| *a == id || *b == id)
        };
        // Family first, then close friends, then playmates, each pair once.
        for pass in 0..3 {
            for (id, character) in characters.iter().filter(|(_, c)| !lazy(c)) {
                if taken(&partners, *id) {
                    continue;
                }
                let found = characters.iter().find(|(other, c)| {
                    let free = other != id && !lazy(c) && !taken(&partners, *other);
                    let bond = cast.bond(*id, *other);
                    free && match pass {
                        0 => character.parent == Some(*other) || c.parent == Some(*id),
                        1 => bond.is_some_and(|bond| bond.warmth >= Band::High),
                        _ => {
                            character.axes.playfulness >= 0.5
                                && bond.is_some_and(|bond| bond.playfulness >= Band::Medium)
                        }
                    }
                });
                if let Some((other, _)) = found {
                    let playing = pass == 2;
                    partners.push((*id, *other, playing));
                    events.push(if playing {
                        Event::Played { a: *id, b: *other }
                    } else {
                        Event::Snuggled { a: *id, b: *other }
                    });
                }
            }
        }
        let seat_of = |id: Id| order.iter().position(|other| *other == id).unwrap_or(0);
        for (id, character) in &characters {
            ground.teleport(*id, SEATS[seat_of(*id) % SEATS.len()]);
            // The snack first, its own way; then the rest, its own way.
            let mut steps: Vec<Step> = character
                .react(Offer::Snack, 0.6, 0, &mut dice)
                .into_iter()
                .map(Step::Beat)
                .collect();
            let partner = partners.iter().find_map(|(a, b, playing)| {
                (*a == *id)
                    .then_some((*b, *playing))
                    .or((*b == *id).then_some((*a, *playing)))
            });
            let mut rest = Vec::new();
            match partner {
                _ if lazy(character) => {
                    rest.push(Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 1.0));
                    let mut doze = Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 600.0);
                    doze.cue = Some(Cue::Sleep);
                    rest.push(doze);
                    events.push(Event::Dozed { who: *id });
                }
                Some((other, false)) => {
                    steps.push(Step::Face(other));
                    let mut fond =
                        Beat::new(ActionKind::PetReaction, ExpressionKind::Affectionate, 2.5);
                    fond.cue = Some(Cue::Heart);
                    rest.push(fond);
                    rest.push(Beat::new(Gesture::Sit, ExpressionKind::Content, 600.0));
                }
                Some((other, true)) => {
                    steps.push(Step::Face(other));
                    rest.extend(character.play_together());
                    rest.push(Beat::new(Gesture::Sit, ExpressionKind::Joy, 600.0));
                }
                None => rest.push(Beat::new(Gesture::Sit, ExpressionKind::Content, 600.0)),
            }
            steps.extend(rest.into_iter().map(Step::Beat));
            ground.direct(*id, steps, now);
        }
        Self {
            since: now,
            over: false,
            events,
        }
    }

    pub fn tick(&mut self, now: f32) {
        if !self.over && now - self.since >= REST_SECS {
            self.over = true;
            self.events.push(Event::Done);
        }
    }

    /// The person would rather be getting on.
    pub fn move_on(&mut self) {
        if !self.over {
            self.over = true;
            self.events.push(Event::Done);
        }
    }

    pub fn over(&self) -> bool {
        self.over
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
}
