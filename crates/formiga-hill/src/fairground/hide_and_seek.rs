//! Hide-and-seek, played by the colony while the person watches. One traveller is "it": it
//! covers its eyes and counts while the rest hide about the fairground, then goes looking.
//!
//! Nobody is written in. Where each one hides and how well, how it gives itself away, and how the
//! seeker goes about looking all come from temperament: the careful pick the tallest cover and
//! keep still, and search one stall after another; the fidgety pick the nearest, keep popping up
//! to look, and dash about when it is their turn; a lazybones dozes off wherever it is, hiding or
//! counting; a grump huffs when the seeker comes near; a show-off hardly hides at all; an
//! explorer can't keep still and changes hiding place halfway. A little one hides near its
//! parent, and a parent who is "it" somehow never finds its little one until last. The longer
//! the search, the more everyone gives away, so nobody stays lost for long.

use super::scenery::HidingPlace;
use crate::actor::Step;
use crate::cast::Id;
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::playground::{Playground, distance};
use formiga_art::ExpressionKind;
use formiga_core::{ActionKind, Gesture, TemperamentKind};

/// Where "it" stands to count: out in the middle, where everyone can see it isn't peeking.
const COUNTING_SPOT: (f32, f32) = (200.0, 150.0);
/// How long the count lasts, and the longest anyone is given to get hidden.
const COUNT_SECS: f32 = 5.0;
const MAX_COUNT_SECS: f32 = 10.0;
/// A sleepy seeker's nap partway through counting.
const DOZE_SECS: f32 = 3.0;
/// How high a fidget pops up over its cover.
const PEEK_LIFT: f32 = 7.0;
const PEEK_SECS: f32 = 0.9;
/// How long the colony celebrates before going back to playing.
const CELEBRATE_SECS: f32 = 4.0;
/// Where the found gather to watch the rest of the search: along the front, clear of the props.
const GATHER: (f32, f32) = (140.0, 202.0);
const GATHER_STEP: f32 = 32.0;
/// How far to one side of the seeker a found hider steps out.
const STEP_OUT: f32 = 20.0;

/// How a traveller plays: where it likes to hide and how it gives itself away, and how it
/// searches when it is "it".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Style {
    Careful,
    Wanderer,
    Grumbly,
    Sleepy,
    Fidget,
    Showy,
}

impl Style {
    pub fn of(character: &Character) -> Self {
        let a = character.axes;
        if character.parent.is_some() || a.impulsiveness > 0.7 {
            return Self::Fidget;
        }
        if a.boldness > 0.82 {
            return Self::Showy;
        }
        match character.kind {
            TemperamentKind::Lazybones => Self::Sleepy,
            TemperamentKind::Grump => Self::Grumbly,
            TemperamentKind::Showoff => Self::Showy,
            TemperamentKind::Explorer => Self::Wanderer,
            TemperamentKind::Troublemaker | TemperamentKind::Oddball => Self::Fidget,
            TemperamentKind::Sweetheart if a.impulsiveness > 0.5 => Self::Fidget,
            _ => Self::Careful,
        }
    }

    /// Seconds between giveaways at the start of a search; they come quicker as it goes on.
    fn patience(self) -> f32 {
        match self {
            Self::Careful => 14.0,
            Self::Fidget => 4.0,
            Self::Sleepy => 7.0,
            Self::Grumbly => 9.0,
            Self::Showy => 5.0,
            Self::Wanderer => 8.0,
        }
    }

    /// How far off a seeker notices a giveaway.
    fn hearing(self) -> f32 {
        match self {
            Self::Careful => 150.0,
            Self::Wanderer => 180.0,
            Self::Grumbly => 120.0,
            Self::Showy => 110.0,
            Self::Fidget => 95.0,
            Self::Sleepy => 80.0,
        }
    }

    /// Whether it dashes from place to place, or walks.
    fn dashes(self) -> bool {
        matches!(self, Self::Fidget | Self::Showy | Self::Wanderer)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Everyone playing about; nobody is hiding.
    Ready,
    /// "It" has its eyes covered while everyone else hides.
    Counting {
        since: f32,
        until: f32,
    },
    Seeking {
        since: f32,
    },
    /// Everyone has been found.
    Over {
        took: f32,
        since: f32,
    },
}

/// Something that happened in the game, for the person watching to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// The seeker nodded off while counting.
    Dozed,
    /// The count is done and the seeker is off.
    Coming,
    Noticed {
        place: usize,
    },
    Found {
        hider: Id,
        place: usize,
    },
    Nobody {
        place: usize,
    },
    AllFound {
        took: f32,
    },
}

#[derive(Clone, Debug)]
struct Hider {
    id: Id,
    place: usize,
    style: Style,
    found: bool,
    asleep: bool,
    moved: bool,
    next_giveaway: f32,
    peeking_until: f32,
    /// How many times it has given itself away this round.
    giveaways: u32,
}

#[derive(Clone, Debug)]
struct Seeker {
    id: Id,
    style: Style,
    /// Places looked behind and found empty, since anyone last moved.
    checked: Vec<usize>,
    /// Where it is on its way to look.
    looking: Option<usize>,
    /// Whether it is going there because it heard something, rather than just trying.
    chasing: bool,
    /// A noise heard while it was busy following another, to try next.
    heard: Option<usize>,
    /// Its own little ones, left till last however loudly they giggle.
    little_ones: Vec<Id>,
    /// Its parent, if it is a little one: who makes sure to be found.
    parent: Option<Id>,
}

pub struct HideAndSeek {
    places: Vec<HidingPlace>,
    phase: Phase,
    seeker: Option<Seeker>,
    hiders: Vec<Hider>,
    events: Vec<Event>,
    round: u64,
    /// Whoever was found first last time: "it" next time, as the rule goes.
    next_it: Option<Id>,
}

impl HideAndSeek {
    pub fn new(places: Vec<HidingPlace>) -> Self {
        Self {
            places,
            phase: Phase::Ready,
            seeker: None,
            hiders: Vec::new(),
            events: Vec::new(),
            round: 0,
            next_it: None,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Who is "it" this round.
    pub fn seeker(&self) -> Option<Id> {
        self.seeker.as_ref().map(|seeker| seeker.id)
    }

    /// Who will be "it" next if nobody says otherwise: whoever was found first last time.
    pub fn next_it(&self) -> Option<Id> {
        self.next_it
    }

    /// How many have been found, of how many are hiding.
    pub fn tally(&self) -> (usize, usize) {
        (
            self.hiders.iter().filter(|h| h.found).count(),
            self.hiders.len(),
        )
    }

    pub fn place_name(&self, place: usize) -> &'static str {
        self.places
            .get(place)
            .map_or("somewhere", |place| place.name)
    }

    /// What has happened since last asked.
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Seconds into the search.
    pub fn searching_for(&self, now: f32) -> f32 {
        match self.phase {
            Phase::Seeking { since } => now - since,
            Phase::Over { took, .. } => took,
            _ => 0.0,
        }
    }

    /// Starts a round with `it` counting, or whoever's turn it is, or whoever is keenest.
    pub fn start(&mut self, ground: &mut Playground, it: Option<Id>, now: f32) {
        let everyone = ground.ids();
        if everyone.len() < 2 {
            return;
        }
        self.round += 1;
        let mut dice = Dice::new(
            self.round.wrapping_mul(0x9e37_79b9) ^ everyone.iter().fold(0, |seed, id| seed ^ id),
        );
        let keenest = || {
            everyone.iter().copied().max_by(|a, b| {
                let keen = |id: Id| {
                    ground
                        .character(id)
                        .map_or(0.0, |c| c.axes.playfulness + c.axes.boldness)
                };
                keen(*a).total_cmp(&keen(*b))
            })
        };
        let it = it
            .or(self.next_it)
            .filter(|id| everyone.contains(id))
            .or_else(keenest)
            .unwrap_or(everyone[0]);
        let Some(character) = ground.character(it).cloned() else {
            return;
        };
        let seeker_style = Style::of(&character);
        self.seeker = Some(Seeker {
            id: it,
            style: seeker_style,
            checked: Vec::new(),
            looking: None,
            chasing: false,
            heard: None,
            little_ones: everyone
                .iter()
                .copied()
                .filter(|id| ground.character(*id).and_then(|c| c.parent) == Some(it))
                .collect(),
            parent: character.parent,
        });

        // Everyone else hides; the careful choose first, so the best cover goes to whoever
        // wants it most.
        let mut order: Vec<(Id, Style)> = everyone
            .iter()
            .copied()
            .filter(|id| *id != it)
            .filter_map(|id| ground.character(id).map(|c| (id, Style::of(c))))
            .collect();
        order.sort_by_key(|(_, style)| *style);
        self.hiders.clear();
        let mut taken: Vec<(Id, usize)> = Vec::new();
        for (id, style) in order {
            if taken.len() == self.places.len() {
                break; // More travellers than hiding places: the rest watch.
            }
            let here = ground.position(id).unwrap_or(COUNTING_SPOT);
            let parent_place = ground
                .character(id)
                .and_then(|c| c.parent)
                .and_then(|parent| taken.iter().find(|(p, _)| *p == parent).map(|(_, at)| *at));
            let place = self.choose_place(style, here, parent_place, &taken, &mut dice);
            taken.push((id, place));
            self.hiders.push(Hider {
                id,
                place,
                style,
                found: false,
                asleep: false,
                moved: false,
                next_giveaway: 0.0,
                peeking_until: 0.0,
                giveaways: 0,
            });
        }

        let reduce_motion = ground.reduce_motion();
        let mut reserved: Vec<Id> = self.hiders.iter().map(|h| h.id).collect();
        reserved.push(it);
        ground.reserve(reserved);
        for hider in &self.hiders {
            let spot = self.places[hider.place].behind;
            let mut steps = Vec::new();
            if reduce_motion {
                ground.teleport(hider.id, spot);
            } else {
                steps.push(Step::Stride { to: spot });
            }
            steps.push(Step::Beat(hiding_pose(hider.style)));
            ground.direct(hider.id, steps, now);
        }

        // "It" goes out to the middle and covers its eyes. A lazybones nods off partway.
        let mut count = COUNT_SECS;
        let mut steps = Vec::new();
        if reduce_motion {
            ground.teleport(it, COUNTING_SPOT);
        } else {
            steps.push(Step::Walk { to: COUNTING_SPOT });
        }
        let cover = |seconds| Beat::new(Gesture::Cover, ExpressionKind::Focused, seconds);
        if seeker_style == Style::Sleepy {
            let mut doze = Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, DOZE_SECS);
            doze.cue = Some(Cue::Sleep);
            steps.extend([
                Step::Beat(cover(2.0)),
                Step::Beat(doze),
                Step::Beat(Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 1.2)),
                Step::Beat(cover(1.5)),
            ]);
            count += DOZE_SECS + 1.2;
            self.events.push(Event::Dozed);
        } else {
            steps.push(Step::Beat(cover(MAX_COUNT_SECS + DOZE_SECS + 2.0)));
        }
        ground.direct(it, steps, now);
        self.phase = Phase::Counting {
            since: now,
            until: now + count,
        };
    }

    fn choose_place(
        &self,
        style: Style,
        here: (f32, f32),
        near: Option<usize>,
        taken: &[(Id, usize)],
        dice: &mut Dice,
    ) -> usize {
        let mut best = (f32::MIN, 0);
        for index in (0..self.places.len()).filter(|i| !taken.iter().any(|(_, at)| at == i)) {
            let place = &self.places[index];
            let far = distance(place.behind, here);
            let away_from_it = distance(place.behind, COUNTING_SPOT);
            let score = dice.range(0.0, 8.0)
                + match (style, near) {
                    (_, Some(parent)) => -distance(place.behind, self.places[parent].behind),
                    (Style::Careful, _) => place.cover * 3.0 + away_from_it * 0.1,
                    (Style::Wanderer, _) => far * 0.3,
                    (Style::Showy, _) => -place.cover * 2.0,
                    (Style::Fidget | Style::Sleepy, _) => -far * 0.3,
                    (Style::Grumbly, _) => away_from_it * 0.2,
                };
            if score > best.0 {
                best = (score, index);
            }
        }
        best.1
    }

    /// Plays the game on: the count, the hiders giving themselves away, the seeker looking.
    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        match self.phase {
            Phase::Ready => {}
            Phase::Counting { since, until } => {
                let settled = self.hiders.iter().all(|h| {
                    ground
                        .position(h.id)
                        .is_some_and(|at| distance(at, self.places[h.place].behind) < 1.0)
                });
                if (now >= until && settled) || now - since >= MAX_COUNT_SECS + DOZE_SECS {
                    self.ready_or_not(ground, now);
                }
            }
            Phase::Seeking { since } => {
                self.give_away(ground, now, since);
                self.search(ground, now);
            }
            Phase::Over { since, .. } => {
                if now - since >= CELEBRATE_SECS {
                    ground.release();
                    self.seeker = None;
                    self.phase = Phase::Ready;
                }
            }
        }
        // Peeks lift a fidget over its cover for a moment.
        for hider in &self.hiders {
            let peeking = !hider.found && now < hider.peeking_until;
            ground.set_lift(hider.id, if peeking { PEEK_LIFT } else { 0.0 });
        }
    }

    /// The count is over: anyone still on their way is simply there, and the seeker sets off.
    fn ready_or_not(&mut self, ground: &mut Playground, now: f32) {
        for hider in &mut self.hiders {
            let spot = self.places[hider.place].behind;
            if ground
                .position(hider.id)
                .is_some_and(|at| distance(at, spot) >= 1.0)
            {
                ground.teleport(hider.id, spot);
                ground.direct(hider.id, vec![Step::Beat(hiding_pose(hider.style))], now);
            }
            hider.next_giveaway = now + hider.style.patience() * 0.5;
        }
        if let Some(seeker) = &self.seeker {
            let mut off = Beat::new(Gesture::Stretch, ExpressionKind::Determined, 0.8);
            off.cue = Some(Cue::Exclaim);
            ground.direct(seeker.id, vec![Step::Beat(off)], now);
        }
        self.events.push(Event::Coming);
        self.phase = Phase::Seeking { since: now };
    }

    fn give_away(&mut self, ground: &mut Playground, now: f32, since: f32) {
        let Some(seeker) = self.seeker.clone() else {
            return;
        };
        let seeker_at = ground.position(seeker.id).unwrap_or(COUNTING_SPOT);
        // Everyone gives more away as the search goes on, so no one stays lost for long.
        let urgency = (1.0 - (now - since) / 60.0).max(0.3);
        for index in 0..self.hiders.len() {
            let hider = self.hiders[index].clone();
            if hider.found || now < hider.next_giveaway {
                continue;
            }
            let place = self.places[hider.place];
            let near = distance(seeker_at, place.behind) < 50.0;
            // A parent makes sure its little one can find it.
            let helping = seeker.parent == Some(hider.id);
            let patience = hider.style.patience() * if helping { 0.4 } else { 1.0 };
            self.hiders[index].next_giveaway = now + patience * urgency;
            self.hiders[index].giveaways += 1;
            match hider.style {
                // A grump can't help grumbling when the seeker comes close.
                Style::Grumbly if near => {
                    let mut huff = Beat::new(Gesture::Huff, ExpressionKind::Grumpy, 1.0);
                    huff.cue = Some(Cue::Huff);
                    let steps = vec![Step::Beat(huff), Step::Beat(hiding_pose(hider.style))];
                    ground.direct(hider.id, steps, now);
                }
                // A fidget pops up for a look, humming to itself.
                Style::Fidget => {
                    self.hiders[index].peeking_until = now + PEEK_SECS;
                    let mut peek = Beat::new(ActionKind::Idle, ExpressionKind::Joy, PEEK_SECS);
                    peek.cue = Some(Cue::Note);
                    let steps = vec![Step::Beat(peek), Step::Beat(hiding_pose(hider.style))];
                    ground.direct(hider.id, steps, now);
                }
                // The careful and the grumbly only shift, and their cover stirs.
                Style::Careful | Style::Grumbly => ground.rustle(hider.place, now),
                Style::Sleepy if !hider.asleep => {
                    self.hiders[index].asleep = true;
                    let mut sleep = Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 600.0);
                    sleep.cue = Some(Cue::Sleep);
                    ground.direct(hider.id, vec![Step::Beat(sleep)], now);
                }
                // Asleep already: the snoring goes on.
                Style::Sleepy => {}
                Style::Showy => {
                    let mut strut = Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.2);
                    strut.cue = Some(Cue::Sparkle);
                    let steps = vec![Step::Beat(strut), Step::Beat(hiding_pose(hider.style))];
                    ground.direct(hider.id, steps, now);
                }
                Style::Wanderer => {
                    // Off to somewhere better, in plain sight for a moment, once.
                    let free = (0..self.places.len()).find(|i| {
                        !self.hiders.iter().any(|h| !h.found && h.place == *i)
                            && seeker.looking != Some(*i)
                            && distance(self.places[*i].behind, seeker_at) > 80.0
                    });
                    if let (false, Some(next)) = (hider.moved, free) {
                        self.hiders[index].moved = true;
                        self.hiders[index].place = next;
                        let to = self.places[next].behind;
                        let mut steps = Vec::new();
                        if ground.reduce_motion() {
                            ground.teleport(hider.id, to);
                        } else {
                            steps.push(Step::Stride { to });
                        }
                        steps.push(Step::Beat(hiding_pose(hider.style)));
                        ground.direct(hider.id, steps, now);
                        // Somewhere already looked might be worth another look now.
                        if let Some(seeker) = &mut self.seeker {
                            seeker.checked.retain(|checked| *checked != next);
                        }
                        continue;
                    }
                    ground.rustle(hider.place, now);
                }
            }
            self.hear(ground, hider.id, now);
        }
    }

    /// The seeker may notice a giveaway, if it is near enough to, and goes straight there.
    fn hear(&mut self, ground: &mut Playground, hider: Id, now: f32) {
        let Some(seeker) = &self.seeker else {
            return;
        };
        let Some(hiding) = self.hiders.iter().find(|h| h.id == hider) else {
            return;
        };
        let place = hiding.place;
        let seeker_at = ground.position(seeker.id).unwrap_or(COUNTING_SPOT);
        let range = seeker.style.hearing()
            * if hiding.style == Style::Showy {
                1.5
            } else {
                1.0
            };
        let others_left = self.hiders.iter().any(|h| !h.found && h.id != hider);
        let pretending = seeker.little_ones.contains(&hider) && others_left;
        if seeker.looking == Some(place)
            || pretending
            || distance(seeker_at, self.places[place].behind) > range
        {
            return;
        }
        // Already following one noise: it will try this one after.
        if seeker.chasing {
            if let Some(seeker) = &mut self.seeker {
                seeker.heard.get_or_insert(place);
            }
            return;
        }
        let mut turn = Beat::new(ActionKind::Idle, ExpressionKind::Curious, 0.5);
        turn.cue = Some(Cue::Exclaim);
        let steps = [Step::FaceX(self.places[place].behind.0), Step::Beat(turn)]
            .into_iter()
            .chain(self.look_steps(ground, place))
            .collect();
        let id = seeker.id;
        ground.direct(id, steps, now);
        if let Some(seeker) = &mut self.seeker {
            seeker.looking = Some(place);
            seeker.chasing = true;
        }
        self.events.push(Event::Noticed { place });
    }

    /// The seeker's own going about: looking behind the place it went to, then choosing where
    /// to try next.
    fn search(&mut self, ground: &mut Playground, now: f32) {
        let Some(seeker) = self.seeker.clone() else {
            return;
        };
        if ground.busy(seeker.id) {
            return;
        }
        if let Some(place) = seeker.looking {
            self.look(ground, place, now);
            return;
        }
        let Some(place) = self.next_place(ground, &seeker) else {
            return;
        };
        let steps = self.look_steps(ground, place);
        ground.direct(seeker.id, steps, now);
        if let Some(seeker) = &mut self.seeker {
            seeker.looking = Some(place);
        }
    }

    /// Where the seeker tries next: along the stalls in order if it is careful, the far side
    /// first if it is an explorer, otherwise whatever is nearest. A parent leaves wherever its
    /// little ones are till last.
    fn next_place(&mut self, ground: &Playground, seeker: &Seeker) -> Option<usize> {
        // Something heard earlier comes first.
        if let Some(place) = seeker.heard {
            if let Some(seeker) = &mut self.seeker {
                seeker.heard = None;
            }
            return Some(place);
        }
        let here = ground.position(seeker.id).unwrap_or(COUNTING_SPOT);
        let hidden_little_ones: Vec<usize> = self
            .hiders
            .iter()
            .filter(|h| !h.found && seeker.little_ones.contains(&h.id))
            .map(|h| h.place)
            .collect();
        let others_left = self
            .hiders
            .iter()
            .any(|h| !h.found && !seeker.little_ones.contains(&h.id));
        let saved_for_last = |i: &usize| others_left && hidden_little_ones.contains(i);
        let mut candidates: Vec<usize> = (0..self.places.len())
            .filter(|i| !seeker.checked.contains(i) && !saved_for_last(i))
            .collect();
        if candidates.is_empty() {
            // Looked everywhere: someone must have moved. Start again.
            if let Some(seeker) = &mut self.seeker {
                seeker.checked.clear();
            }
            candidates = (0..self.places.len())
                .filter(|i| !saved_for_last(i))
                .collect();
        }
        let score = |index: &usize| {
            let at = self.places[*index].behind;
            match seeker.style {
                Style::Careful => -at.0,
                Style::Wanderer => distance(at, here),
                _ => -distance(at, here),
            }
        };
        candidates
            .into_iter()
            .max_by(|a, b| score(a).total_cmp(&score(b)))
    }

    /// Going over to a place and peering behind it.
    fn look_steps(&self, ground: &Playground, place: usize) -> Vec<Step> {
        let Some(seeker) = &self.seeker else {
            return Vec::new();
        };
        let spot = self.places[place].out_front;
        let mut steps = Vec::new();
        if ground.reduce_motion() {
            // A tableau: it is simply there, looking, once the look is over.
        } else if seeker.style.dashes() {
            steps.push(Step::Stride { to: spot });
        } else {
            steps.push(Step::Walk { to: spot });
        }
        steps.push(Step::FaceX(self.places[place].behind.0 + 0.5));
        steps.push(Step::Beat(Beat::new(
            Gesture::Peek,
            ExpressionKind::Curious,
            0.9,
        )));
        steps
    }

    /// The seeker has looked behind `place`: whoever is there is found, and comes out its own way.
    fn look(&mut self, ground: &mut Playground, place: usize, now: f32) {
        let Some(seeker) = self.seeker.clone() else {
            return;
        };
        if let Some(s) = &mut self.seeker {
            s.looking = None;
            s.chasing = false;
            if s.heard == Some(place) {
                s.heard = None;
            }
        }
        if ground.reduce_motion() {
            ground.teleport(seeker.id, self.places[place].out_front);
        }
        ground.rustle(place, now);
        let Some(index) = self
            .hiders
            .iter()
            .position(|h| !h.found && h.place == place)
        else {
            if let Some(s) = &mut self.seeker {
                s.checked.push(place);
            }
            let shrug = empty_handed(seeker.style);
            ground.direct(seeker.id, shrug.into_iter().map(Step::Beat).collect(), now);
            self.events.push(Event::Nobody { place });
            return;
        };
        let Phase::Seeking { since } = self.phase else {
            return;
        };
        let found_before = self.tally().0;
        self.hiders[index].found = true;
        // Nobody else is behind there now.
        if let Some(s) = &mut self.seeker {
            s.checked.push(place);
        }
        if found_before == 0 {
            self.next_it = Some(self.hiders[index].id);
        }
        let hider = self.hiders[index].clone();
        let character = ground.character(hider.id).cloned();
        let seeker_character = ground.character(seeker.id).cloned();
        self.events.push(Event::Found {
            hider: hider.id,
            place,
        });

        // The seeker points it out; the hider steps out beside it.
        let mut spotted = Beat::new(Gesture::Reach, ExpressionKind::Joy, 0.8);
        spotted.cue = Some(Cue::Exclaim);
        let out = self.places[place].out_front;
        let side = if out.0 > 192.0 { -STEP_OUT } else { STEP_OUT };
        let beside = (out.0 + side, out.1 + 2.0);
        let mut steps = Vec::new();
        if ground.reduce_motion() {
            ground.teleport(hider.id, beside);
        } else {
            steps.push(Step::Stride { to: beside });
        }
        steps.push(Step::Face(seeker.id));
        steps.extend(
            found_reaction(hider.style, hider.asleep, character.as_ref())
                .into_iter()
                .map(Step::Beat),
        );

        if self.hiders.iter().all(|h| h.found) {
            let took = now - since;
            // The seeker cheers, the last one found joins in, and so does everyone watching.
            let cheer = |c: Option<&Character>| {
                c.map_or_else(
                    || Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.6),
                    |c| c.celebrate(1.6),
                )
            };
            steps.push(Step::Beat(cheer(character.as_ref())));
            let seeker_steps = vec![
                Step::Beat(spotted),
                Step::Beat(cheer(seeker_character.as_ref())),
            ];
            ground.direct(seeker.id, seeker_steps, now);
            for other in self.hiders.iter().filter(|h| h.id != hider.id) {
                if let Some(c) = ground.character(other.id) {
                    let steps = vec![Step::Face(seeker.id), Step::Beat(c.celebrate(1.6))];
                    ground.direct(other.id, steps, now);
                }
            }
            self.events.push(Event::AllFound { took });
            self.phase = Phase::Over { took, since: now };
        } else {
            ground.direct(seeker.id, vec![Step::Beat(spotted)], now);
            // Then over to the front, to watch the rest of the search.
            let gather = (GATHER.0 + (found_before % 7) as f32 * GATHER_STEP, GATHER.1);
            if !ground.reduce_motion() {
                steps.push(Step::Stride { to: gather });
            }
            steps.push(Step::Face(seeker.id));
            steps.push(Step::Beat(Beat::new(
                Gesture::Watch,
                ExpressionKind::Curious,
                600.0,
            )));
        }
        ground.direct(hider.id, steps, now);
    }

    /// Calls the game off: everyone comes out, pleased with themselves, and goes back to playing.
    pub fn stop(&mut self, ground: &mut Playground, now: f32) {
        let ids = self
            .hiders
            .iter()
            .map(|h| h.id)
            .chain(self.seeker.as_ref().map(|s| s.id));
        for id in ids {
            if let Some(c) = ground.character(id) {
                ground.direct(id, vec![Step::Beat(c.celebrate(1.0))], now);
            }
            ground.set_lift(id, 0.0);
        }
        ground.release();
        self.hiders.clear();
        self.seeker = None;
        self.events.clear();
        self.phase = Phase::Ready;
    }
}

/// How a hider waits: low and still if it can manage it, standing proud if it cannot be bothered.
fn hiding_pose(style: Style) -> Beat {
    match style {
        Style::Showy => Beat::new(ActionKind::Idle, ExpressionKind::Smug, 600.0),
        Style::Grumbly => Beat::new(Gesture::Crouch, ExpressionKind::Grumpy, 600.0),
        Style::Fidget => Beat::new(Gesture::Crouch, ExpressionKind::Joy, 600.0),
        _ => Beat::new(Gesture::Crouch, ExpressionKind::Focused, 600.0),
    }
}

/// Nobody there: each seeker takes it its own way.
fn empty_handed(style: Style) -> Vec<Beat> {
    match style {
        Style::Grumbly => {
            let mut stomp = Beat::new(Gesture::Stomp, ExpressionKind::Grumpy, 0.8);
            stomp.cue = Some(Cue::Huff);
            vec![stomp]
        }
        Style::Showy => vec![Beat::new(Gesture::Strut, ExpressionKind::Smug, 0.8)],
        Style::Sleepy => vec![Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 1.0)],
        Style::Fidget => vec![Beat::new(ActionKind::Idle, ExpressionKind::Curious, 0.3)],
        Style::Careful | Style::Wanderer => {
            vec![Beat::new(Gesture::Watch, ExpressionKind::Focused, 0.6)]
        }
    }
}

/// Found! Each hider takes it its own way.
fn found_reaction(style: Style, asleep: bool, character: Option<&Character>) -> Vec<Beat> {
    let celebrate = |seconds| {
        character.map_or_else(
            || Beat::new(Gesture::Cheer, ExpressionKind::Joy, seconds),
            |c| c.celebrate(seconds),
        )
    };
    let with_cue = |mut beat: Beat, cue| {
        beat.cue = Some(cue);
        beat
    };
    if asleep {
        return vec![
            Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 1.2),
            Beat::new(ActionKind::Idle, ExpressionKind::Sleepy, 0.8),
        ];
    }
    match style {
        Style::Fidget => vec![celebrate(1.0)],
        Style::Careful => vec![
            with_cue(
                Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.5),
                Cue::Exclaim,
            ),
            Beat::new(ActionKind::Idle, ExpressionKind::Content, 0.8),
        ],
        Style::Grumbly => vec![
            with_cue(
                Beat::new(Gesture::Huff, ExpressionKind::Grumpy, 1.0),
                Cue::Huff,
            ),
            Beat::new(ActionKind::Idle, ExpressionKind::Smug, 0.6),
        ],
        Style::Showy => vec![with_cue(
            Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.2),
            Cue::Sparkle,
        )],
        Style::Wanderer | Style::Sleepy => vec![
            with_cue(
                Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.4),
                Cue::Exclaim,
            ),
            celebrate(0.9),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::fairground;

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    /// Runs the game until `to`, gathering what happened.
    fn play(
        ground: &mut Playground,
        game: &mut HideAndSeek,
        cast: &Cast,
        from: f32,
        to: f32,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            game.tick(ground, now);
            events.extend(game.take_events());
        }
        events
    }

    /// The fairground, and its game of hide-and-seek.
    fn open(cast: &Cast) -> (Playground, HideAndSeek) {
        let (ground, games) = fairground::open(cast, 0.0);
        (ground, games.hide_and_seek)
    }

    fn settled(cast: &Cast) -> (Playground, HideAndSeek) {
        let (mut ground, mut game) = open(cast);
        play(&mut ground, &mut game, cast, 0.0, 10.0);
        (ground, game)
    }

    fn found(events: &[Event]) -> Vec<Id> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Found { hider, .. } => Some(*hider),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn everyone_but_it_hides_somewhere_different() {
        let cast = sample();
        let (mut ground, mut game) = settled(&cast);
        game.start(&mut ground, None, 10.0);
        let it = game.seeker().unwrap();
        assert!(game.hiders.iter().all(|h| h.id != it));
        assert_eq!(game.hiders.len(), cast.members.len() - 1);
        let mut places: Vec<usize> = game.hiders.iter().map(|h| h.place).collect();
        places.sort();
        places.dedup();
        assert_eq!(places.len(), game.hiders.len());
        play(&mut ground, &mut game, &cast, 10.0, 26.0);
        assert!(matches!(
            game.phase(),
            Phase::Seeking { .. } | Phase::Over { .. }
        ));
    }

    #[test]
    fn the_seeker_finds_everyone_and_the_first_found_is_it_next() {
        let cast = sample();
        let (mut ground, mut game) = settled(&cast);
        game.start(&mut ground, None, 10.0);
        let events = play(&mut ground, &mut game, &cast, 10.0, 200.0);
        let found = found(&events);
        assert_eq!(found.len(), cast.members.len() - 1, "{events:?}");
        let took = events.iter().find_map(|event| match event {
            Event::AllFound { took } => Some(*took),
            _ => None,
        });
        assert!(took.is_some_and(|took| took < 120.0), "took {took:?}");
        assert_eq!(game.phase(), Phase::Ready, "everyone went back to playing");
        assert_eq!(game.next_it(), Some(found[0]));
    }

    #[test]
    fn every_traveller_can_be_it_and_finds_everyone() {
        let cast = sample();
        for member in &cast.members {
            let (mut ground, mut game) = settled(&cast);
            game.start(&mut ground, Some(member.id), 10.0);
            assert_eq!(game.seeker(), Some(member.id));
            let events = play(&mut ground, &mut game, &cast, 10.0, 200.0);
            assert_eq!(
                found(&events).len(),
                cast.members.len() - 1,
                "{} as it: {events:?}",
                member.name
            );
        }
    }

    #[test]
    fn a_parent_finds_its_little_one_last() {
        let cast = sample();
        let Some(parent) = cast.members.iter().find_map(|member| member.parent()) else {
            return;
        };
        let little_ones: Vec<Id> = cast
            .members
            .iter()
            .filter(|member| member.parent() == Some(parent))
            .map(|member| member.id)
            .collect();
        let (mut ground, mut game) = settled(&cast);
        game.start(&mut ground, Some(parent), 10.0);
        let found = found(&play(&mut ground, &mut game, &cast, 10.0, 200.0));
        let tail = &found[found.len() - little_ones.len()..];
        assert!(
            little_ones.iter().all(|id| tail.contains(id)),
            "found in order {found:?}, little ones {little_ones:?}"
        );
    }

    #[test]
    fn temperament_decides_how_each_one_plays() {
        let cast = sample();
        let ground = fairground::open(&cast, 0.0).0;
        let mut styles: Vec<Style> = ground
            .ids()
            .into_iter()
            .map(|id| Style::of(ground.character(id).unwrap()))
            .collect();
        styles.sort();
        styles.dedup();
        assert!(
            styles.len() >= 3,
            "the sample colony all plays alike: {styles:?}"
        );
        for id in ground.ids() {
            let character = ground.character(id).unwrap();
            if character.parent.is_some() {
                assert_eq!(
                    Style::of(character),
                    Style::Fidget,
                    "little ones can't keep still"
                );
            }
        }
    }

    #[test]
    fn a_long_search_gives_everyone_away_in_the_end() {
        let cast = sample();
        let (mut ground, mut game) = settled(&cast);
        game.start(&mut ground, None, 10.0);
        play(&mut ground, &mut game, &cast, 10.0, 24.0);
        // Nobody finds them: the seeker is kept covering its eyes the whole time.
        let it = game.seeker().unwrap();
        let busy = Beat::new(Gesture::Cover, ExpressionKind::Focused, 600.0);
        ground.direct(it, vec![Step::Beat(busy)], 24.0);
        if let Some(seeker) = &mut game.seeker {
            seeker.looking = None;
            seeker.style = Style::Sleepy;
        }
        ground.teleport(it, (380.0, 60.0));
        play(&mut ground, &mut game, &cast, 24.0, 84.0);
        for hider in game.hiders.iter().filter(|h| !h.found) {
            assert!(hider.giveaways >= 2, "{:?} stayed lost", hider.style);
        }
    }

    #[test]
    fn with_reduced_motion_the_game_still_plays_out() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut game) = settled(&cast);
        game.start(&mut ground, None, 10.0);
        let events = play(&mut ground, &mut game, &cast, 10.0, 200.0);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::AllFound { .. }))
        );
    }

    #[test]
    fn stopping_lets_everyone_out() {
        let cast = sample();
        let (mut ground, mut game) = open(&cast);
        game.start(&mut ground, None, 1.0);
        game.stop(&mut ground, 2.0);
        assert_eq!(game.phase(), Phase::Ready);
        assert_eq!(game.tally(), (0, 0));
        assert_eq!(game.seeker(), None);
    }
}
