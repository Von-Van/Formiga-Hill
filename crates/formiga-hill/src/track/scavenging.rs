//! A scavenge along the old track: one outing with a companion or two, until the light goes or the
//! person takes the basket home.
//!
//! The person plays. Choosing a heap sends the party to it, and walking costs a little light.
//! At a heap the person chooses who does what (each companion's way of lifting, and a peek or a
//! squeeze for those who can), then what to do it to. Everything costs light: lifting, peeking,
//! squeezing. Lifting something opens up whatever was under it or in it, and anything found goes
//! into the basket; but pulling something out from under others brings them tumbling down onto
//! whatever was under it, burying it again and cracking it if it breaks. So the way to go is from
//! the top down, unless someone gentle eases things out, or a little one squeezes into a gap. The
//! good things are underneath; the rare ones glint only as the light starts to go; and now and
//! then a torn map turns up, to be followed another day.
//!
//! Who came along matters (see `crew`): a strong one or two together for the heavy things, a
//! patient one easing things out, an impulsive one yanking them, a curious one peeking first, a
//! little one squeezing in, and one that looks things over knowing what is in every sack, crate
//! and tin. And what lies in the heaps leans towards who came (see `finds`).

use super::crew::{self, Crew, Hand};
use super::heap::{self, Heap, Hoard, How, Place, Refusal, Stocking, Stuff};
use super::stuff::{self, Motion, map_icon, scene_rect};
use super::{SITES, Site};
use crate::actor::Step;
use crate::cast::Id;
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::finds::{self, Tier, art};
use crate::paint::{blit, mix, put, rect, rgb, rgba};
use crate::playground::{Playground, Prop, distance};
use crate::woods::Influence;
use crate::woods::rummage::{delighted, dim};
use formiga_art::{Canvas, ExpressionKind, Rgba};
use formiga_core::{ActionKind, Gesture, HabitCue, TemperamentKind};

/// The light an outing starts with, before the Hilltop lends any.
pub const LIGHT: f32 = 100.0;
/// How many finds the basket holds.
pub const BASKET: usize = 6;
/// What walking costs per pixel, for a middling companion; and lifting, easing something out,
/// yanking, peeking and squeezing in. Pulling something out from under others costs as much
/// again for everything on it, pulled against, and a yank half as much; only easing it out
/// costs the same however much is on it. Something heavy costs more, however it is lifted.
const WALK_COST: f32 = 0.035;
const LIFT_COST: f32 = 3.0;
const GENTLE_COST: f32 = 4.5;
const YANK_COST: f32 = 1.5;
const HEAVY_COST: f32 = 1.6;
const PEEK_COST: f32 = 1.0;
const SQUEEZE_COST: f32 = 2.0;
/// Clearing what came down when the heap shifts, for each thing that came down.
const TUMBLE_COST: f32 = 1.0;
/// Below this much light, rare things start to glint.
pub const DUSK: f32 = 55.0;
/// How long each thing takes to do.
const LIFT_SECS: f32 = 0.8;
const GENTLE_SECS: f32 = 1.3;
const YANK_SECS: f32 = 0.5;
const HEAVE_SECS: f32 = 1.4;
const PEEK_SECS: f32 = 0.9;
const SQUEEZE_SECS: f32 = 1.6;
/// How long a glint shows, something found is held up, and a crack shows.
const SIGN_SECS: f32 = 1.2;
const SHOW_SECS: f32 = 1.4;
const CRACK_SECS: f32 = 1.2;
/// How far beside the one working a heap a second companion stands.
const BESIDE: f32 = 20.0;
/// The most torn maps worth holding at once: past this, no more turn up.
pub const MAPS_HELD: usize = 5;

/// The light lifting something costs, `how` it is lifted, heavy or not, with `load` things on it.
pub fn cost(how: How, heavy: bool, load: usize) -> f32 {
    let pulled = (1 + load) as f32;
    let cost = match how {
        How::Plain => LIFT_COST * pulled,
        How::Yank => YANK_COST * pulled,
        How::Gentle => GENTLE_COST,
    };
    if heavy { cost * HEAVY_COST } else { cost }
}

/// How likely a torn map is to be somewhere in the heaps, by how many the colony holds already.
pub fn map_chance(held: usize) -> f32 {
    match held {
        0 => 0.35,
        1 => 0.22,
        2 => 0.12,
        held if held < MAPS_HELD => 0.06,
        _ => 0.0,
    }
}

/// What an outing sets out with: the same shape as every Woods outing's, so an expedition could
/// one day hand one on to the next, with how many maps the colony holds already.
pub struct Outset {
    /// Who came along; the first leads.
    pub party: Vec<Id>,
    /// Outings to the old track in a row that brought nothing new home.
    pub drought: u32,
    /// Whether the two who came are close friends.
    pub close_pair: bool,
    /// What the Hilltop lends the Woods: light, and the rare things sooner.
    pub influence: Influence,
    /// Torn maps the colony holds, not yet followed.
    pub maps_held: usize,
    pub seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// The companions are walking up the track.
    Arriving {
        since: f32,
    },
    /// On the track, deciding which heap.
    Choosing,
    Going {
        heap: usize,
    },
    /// At a heap, ready.
    At {
        heap: usize,
    },
    /// At a heap, doing something, until then.
    Working {
        heap: usize,
        until: f32,
    },
    Leaving {
        since: f32,
        why: Ending,
    },
    /// Everyone has gone home; the basket is ready to be kept.
    Over,
}

/// Why an outing ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    Dusk,
    Chose,
}

/// Something that happened, for the person to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Lifted {
        who: Id,
        stuff: Stuff,
        how: How,
    },
    /// Two lifted something heavy together.
    Together {
        who: Id,
        with: Id,
        stuff: Stuff,
    },
    TooHeavy {
        who: Id,
        stuff: Stuff,
    },
    /// The heap shifted: this many things came down.
    Tumbled {
        count: usize,
    },
    /// Something breakable cracked.
    Cracked,
    Got {
        find: &'static str,
    },
    /// A torn map!
    Map {
        who: Id,
    },
    Peeked {
        who: Id,
        stuff: Stuff,
        /// What was there, the first of it, and how many things.
        first: Option<&'static str>,
        count: usize,
    },
    Squeezed {
        who: Id,
        count: usize,
    },
    NoGap {
        who: Id,
    },
    /// One that looks things over looked over what came out.
    Looked {
        who: Id,
        find: &'static str,
    },
    /// Found with nothing under it or in it.
    Nothing {
        stuff: Stuff,
    },
    BasketFull,
    PutBack {
        find: &'static str,
    },
    /// The light is going: rare things glint now.
    Dusk,
    Leaving(Ending),
}

/// What a careful hand would do next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Go(usize),
    /// Choose this hand, then do it to this item.
    Act {
        hand: usize,
        item: usize,
    },
    /// Pick up this hidden thing lying in the open.
    Take(usize),
    /// Put back what is in this slot of the basket, to make room for something better.
    PutBack(usize),
    Home,
    Wait,
}

/// A glint where something lies hidden, now and then.
#[derive(Clone, Copy, Debug, Default)]
struct Sign {
    next: f32,
    until: f32,
}

/// What a find looks like, glinting from where it is hidden.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shine {
    /// Metal: a warm glint.
    Glint,
    /// China and glass: a cool gleam.
    Gleam,
    /// Paper, straw or wood: a corner showing.
    Corner(u32),
}

fn shine(what: Hoard) -> Shine {
    match what {
        Hoard::Map => Shine::Corner(0xe6d0a4),
        Hoard::Find(find) => match find.id {
            "china_teacup" | "cracked_teacup" | "ship_in_a_bottle" => Shine::Gleam,
            "straw_hat" => Shine::Corner(0xd8c070),
            "carved_sign" | "cuckoo_clock" => Shine::Corner(0xa8885a),
            _ => Shine::Glint,
        },
    }
}

pub struct Scavenge {
    /// How dark the hour has made the track already, so the outing's own dusk only darkens it
    /// past that.
    hour_dark: f32,
    crew: Crew,
    sites: Vec<Site>,
    heaps: Vec<Heap>,
    motion: Vec<Motion>,
    signs: Vec<Vec<Sign>>,
    /// Whether each hidden thing has glinted yet, for a careful hand that watches for them.
    glinted: Vec<Vec<bool>>,
    phase: Phase,
    light: f32,
    full: f32,
    /// Below this much light, rare things glint.
    dusk: f32,
    basket: Vec<&'static str>,
    /// Torn maps found, by who found each.
    maps: Vec<Id>,
    events: Vec<Event>,
    dice: Dice,
    reduce_motion: bool,
    /// Which of the party's hands the person has chosen.
    hand: usize,
    /// Where the walk to the next heap starts from.
    from: (f32, f32),
    walk_cost: f32,
    /// Something just found, held up: what, where in the scene, and since when.
    shown: Option<(Hoard, (i32, i32), f32)>,
    /// Cracks showing where something broke: where, and since when.
    cracks: Vec<((i32, i32), f32)>,
    dusk_told: bool,
    /// Where the party goes when it goes home.
    way_home: (f32, f32),
}

impl Scavenge {
    /// Sets out with the party, already walking up the track in `ground`. `found_before` says
    /// which finds the colony has already.
    pub fn new(
        ground: &mut Playground,
        outset: Outset,
        found_before: impl Fn(&str) -> bool,
        now: f32,
    ) -> Self {
        let Outset {
            party,
            drought,
            close_pair,
            influence,
            maps_held,
            seed,
        } = outset;
        let characters: Vec<Character> = party
            .iter()
            .filter_map(|id| ground.character(*id).cloned())
            .collect();
        let crew = Crew::new(party, characters, close_pair);
        let mut dice = Dice::new(seed);
        let sites = SITES.to_vec();
        let mut heaps: Vec<Heap> = sites
            .iter()
            .map(|site| Heap::build(&site.recipe, site.width, &mut dice))
            .collect();
        let refs: Vec<&Character> = crew.characters.iter().collect();
        let stocking = Stocking {
            party: &refs,
            found_before: &found_before,
            drought,
            map: map_chance(maps_held),
            inspector: crew.first(crew::inspector).is_some(),
        };
        heap::stock(&mut heaps, &stocking, &mut dice);
        let full = LIGHT + influence.light;
        let mut scavenge = Self::with(ground, crew, sites, heaps, (full, full), dice, now);
        scavenge.dusk = DUSK + influence.earlier;
        scavenge
    }

    /// A heap at the end of a wrong turn on a treasure hunt: the party is already beside it, with
    /// the hunt's light and basket.
    pub fn nook(
        ground: &mut Playground,
        crew: Crew,
        (light, full): (f32, f32),
        basket: Vec<&'static str>,
        found_before: impl Fn(&str) -> bool,
        seed: u64,
        now: f32,
    ) -> Self {
        let mut dice = Dice::new(seed);
        let site = super::NOOK;
        let mut heaps = vec![Heap::build(&site.recipe, site.width, &mut dice)];
        let refs: Vec<&Character> = crew.characters.iter().collect();
        let stocking = Stocking {
            party: &refs,
            found_before: &found_before,
            drought: 0,
            map: 0.0,
            inspector: crew.first(crew::inspector).is_some(),
        };
        heap::stock(&mut heaps, &stocking, &mut dice);
        let mut nook = Self::with(ground, crew, vec![site], heaps, (light, full), dice, now);
        nook.basket = basket;
        nook.from = site.stand;
        nook.way_home = (site.stand.0, 230.0);
        nook.choose(ground, 0, now);
        nook
    }

    fn with(
        ground: &mut Playground,
        crew: Crew,
        sites: Vec<Site>,
        heaps: Vec<Heap>,
        (light, full): (f32, f32),
        mut dice: Dice,
        now: f32,
    ) -> Self {
        let signs = heaps
            .iter()
            .map(|heap| {
                heap.hidden
                    .iter()
                    .map(|_| Sign {
                        next: now + 2.0 + dice.range(0.0, 3.0),
                        until: 0.0,
                    })
                    .collect()
            })
            .collect();
        // The slowest sets the pace.
        let liveliness = crew
            .characters
            .iter()
            .map(|c| c.axes.energy)
            .fold(1.0, f32::min);
        // Out here for the heaps, not to wander off.
        ground.reserve(crew.party.clone());
        Self {
            hour_dark: 0.0,
            motion: vec![Motion::default(); heaps.len()],
            glinted: heaps
                .iter()
                .map(|heap| vec![false; heap.hidden.len()])
                .collect(),
            signs,
            sites,
            heaps,
            phase: Phase::Arriving { since: now },
            light,
            full,
            dusk: DUSK,
            basket: Vec::new(),
            maps: Vec::new(),
            events: Vec::new(),
            dice,
            reduce_motion: ground.reduce_motion(),
            hand: 0,
            from: (-24.0, 200.0),
            walk_cost: WALK_COST / (0.8 + 0.4 * liveliness),
            shown: None,
            cracks: Vec::new(),
            dusk_told: false,
            way_home: (-30.0, 202.0),
            crew,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn party(&self) -> &[Id] {
        &self.crew.party
    }

    pub fn crew(&self) -> &Crew {
        &self.crew
    }

    pub fn basket(&self) -> &[&'static str] {
        &self.basket
    }

    /// The torn maps found, by who found each.
    pub fn maps(&self) -> &[Id] {
        &self.maps
    }

    /// How much of the light is left, from 0 to 1.
    pub fn light_left(&self) -> f32 {
        (self.light / self.full).clamp(0.0, 1.0)
    }

    /// The light left, and the light the outing started with, for handing on.
    pub fn light(&self) -> (f32, f32) {
        (self.light, self.full)
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// How dark the hour has made the track already: the outing's dusk adds only what is more.
    pub fn set_hour_dark(&mut self, darkness: f32) {
        self.hour_dark = darkness;
    }

    pub fn heap(&self, index: usize) -> Option<&Heap> {
        self.heaps.get(index)
    }

    pub fn site(&self, index: usize) -> Option<&Site> {
        self.sites.get(index)
    }

    #[cfg(test)]
    pub fn heap_count(&self) -> usize {
        self.heaps.len()
    }

    /// The heap the party is at, if it is at one.
    pub fn at(&self) -> Option<usize> {
        match self.phase {
            Phase::At { heap } | Phase::Working { heap, .. } => Some(heap),
            _ => None,
        }
    }

    /// Everything the party can do at a heap.
    pub fn hands(&self) -> Vec<Hand> {
        self.crew.hands()
    }

    /// The hand the person has chosen.
    pub fn hand(&self) -> usize {
        self.hand
    }

    pub fn set_hand(&mut self, hand: usize) {
        if hand < self.hands().len() {
            self.hand = hand;
        }
    }

    /// The heap a point is on or near.
    pub fn heap_at(&self, x: f32, y: f32) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .filter(|(index, site)| {
                let (left, right) = self.heaps[*index].span();
                let top = self.heaps[*index]
                    .items
                    .iter()
                    .filter(|item| !item.lifted)
                    .map(|item| item.top())
                    .max()
                    .unwrap_or(4);
                x >= (site.left + left - 6) as f32
                    && x <= (site.left + right + 6) as f32
                    && y >= (site.ground - top - 8) as f32
                    && y <= (site.ground + 6) as f32
            })
            .map(|(index, _)| index)
            .next()
    }

    /// The thing in a heap under a point: the front-most, last laid on.
    pub fn item_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let (x, y) = (x.floor() as i32, y.floor() as i32);
        for (heap, site) in self.sites.iter().enumerate() {
            let found = self.heaps[heap]
                .items
                .iter()
                .enumerate()
                .rev()
                .filter(|(_, item)| !item.lifted)
                .find(|(_, item)| {
                    let (left, top, w, h) = scene_rect(site, item);
                    x >= left - 1 && x <= left + w && y >= top - 1 && y <= top + h
                });
            if let Some((item, _)) = found {
                return Some((heap, item));
            }
        }
        None
    }

    /// Something lying out in the open under a point: its heap, and which hidden thing.
    pub fn open_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        for (heap, site) in self.sites.iter().enumerate() {
            for (index, hidden) in self.heaps[heap].hidden.iter().enumerate() {
                if hidden.place != Place::Open {
                    continue;
                }
                let middle = (
                    (site.left + hidden.at) as f32,
                    (site.ground - hidden.lies - 4) as f32,
                );
                if distance((x, y), middle) <= 6.0 {
                    return Some((heap, index));
                }
            }
        }
        None
    }

    /// What to call something in a heap, as the pointer is over it: what it is, and anything
    /// worth knowing about it.
    pub fn describe(&self, heap: usize, item: usize) -> String {
        let Some(at) = self.heaps.get(heap) else {
            return String::new();
        };
        let Some(thing) = at.items.get(item) else {
            return String::new();
        };
        let mut words = vec![thing.stuff.name().to_owned()];
        if thing.stuff.heavy() {
            words.push("heavy".to_owned());
        }
        match at.resting_on(item).len() {
            0 => {}
            1 => words.push("something rests on it".to_owned()),
            count => words.push(format!("{count} things rest on it")),
        }
        let there = at.hidden_at(item);
        let known: Vec<&str> = there
            .iter()
            .filter(|&&hidden| at.hidden[hidden].known)
            .map(|&hidden| match at.hidden[hidden].what {
                Hoard::Find(find) => find.name,
                Hoard::Map => "a torn map",
            })
            .collect();
        if !known.is_empty() {
            words.push(known.join(", ").to_lowercase());
        } else if thing.peeked {
            words.push("nothing there".to_owned());
        } else if self.crew.first(crew::little).is_some() && at.gap(item) {
            words.push("a gap under it".to_owned());
        }
        words.join(" \u{b7} ")
    }

    /// The person chose a heap: the party goes over to it.
    pub fn choose(&mut self, ground: &mut Playground, heap: usize, now: f32) {
        let ready = match self.phase {
            Phase::Choosing => true,
            Phase::At { heap: here } | Phase::Going { heap: here } => here != heap,
            _ => false,
        };
        let Some(site) = self.sites.get(heap).copied() else {
            return;
        };
        if !ready {
            return;
        }
        self.light -= distance(self.from, site.stand) * self.walk_cost;
        self.from = site.stand;
        ground.reserve(self.crew.party.clone());
        let middle = (site.left + site.width / 2) as f32;
        for (index, id) in self.crew.party.clone().into_iter().enumerate() {
            let to = if index == 0 {
                site.stand
            } else {
                let side = if site.stand.0 < middle {
                    -BESIDE
                } else {
                    BESIDE
                };
                ground.beside_point((site.stand.0, site.stand.1 + 3.0), side)
            };
            ground.direct(id, vec![Step::Walk { to }, Step::FaceX(middle)], now);
        }
        self.phase = Phase::Going { heap };
    }

    /// Does whatever the chosen hand does to something in the heap the party is at.
    pub fn act(&mut self, ground: &mut Playground, item: usize, now: f32) {
        let Phase::At { heap } = self.phase else {
            return;
        };
        let Some(hand) = self.hands().get(self.hand).copied() else {
            return;
        };
        let Some(thing) = self.heaps[heap].items.get(item).cloned() else {
            return;
        };
        if thing.lifted {
            return;
        }
        let site = self.sites[heap];
        let (left, top, w, h) = scene_rect(&site, &thing);
        let face = (left + w / 2) as f32;
        let seconds = match hand {
            Hand::Lift(who) => {
                let Some((how, lifters)) = self.crew.lift(who, thing.stuff.heavy()) else {
                    self.events.push(Event::TooHeavy {
                        who,
                        stuff: thing.stuff,
                    });
                    let mut strain = Beat::new(Gesture::Heave, ExpressionKind::Determined, 0.8);
                    strain.cue = Some(Cue::Huff);
                    ground.direct(who, vec![Step::FaceX(face), Step::Beat(strain)], now);
                    return;
                };
                let load = self.heaps[heap].above(item).len();
                self.light -= cost(how, thing.stuff.heavy(), load);
                let Ok(shift) = self.heaps[heap].lift(item, how) else {
                    return;
                };
                self.events.push(Event::Lifted {
                    who,
                    stuff: thing.stuff,
                    how,
                });
                if let [_, with] = lifters.as_slice() {
                    self.events.push(Event::Together {
                        who,
                        with: *with,
                        stuff: thing.stuff,
                    });
                }
                let motion = &mut self.motion[heap];
                motion.lifting.push((thing.clone(), now));
                for &(at, from) in &shift.fell {
                    motion.falling.push((at, from, now));
                    let landed = &self.heaps[heap].items[at];
                    motion
                        .puffs
                        .push((landed.x + landed.w / 2, landed.y, now + stuff::FALL_SECS));
                }
                if !shift.fell.is_empty() && how != How::Gentle {
                    self.light -= TUMBLE_COST * shift.fell.len() as f32;
                    self.events.push(Event::Tumbled {
                        count: shift.fell.len(),
                    });
                }
                for &hidden in &shift.cracked {
                    let what = &self.heaps[heap].hidden[hidden];
                    let lies = self.heaps[heap].level(hidden);
                    self.cracks
                        .push(((site.left + what.at, site.ground - lies - 4), now));
                    self.events.push(Event::Cracked);
                }
                let beat = if thing.stuff.heavy() {
                    Beat::new(Gesture::Heave, ExpressionKind::Determined, HEAVE_SECS)
                } else if top + h / 2 < site.ground - 14 {
                    Beat::new(Gesture::Reach, ExpressionKind::Focused, LIFT_SECS)
                } else {
                    Beat::new(Gesture::Crouch, ExpressionKind::Focused, LIFT_SECS)
                };
                for lifter in &lifters {
                    ground.direct(*lifter, vec![Step::FaceX(face), Step::Beat(beat)], now);
                }
                let found = shift.revealed.clone();
                if found.is_empty() && self.heaps[heap].hidden_at(item).is_empty() {
                    self.events.push(Event::Nothing { stuff: thing.stuff });
                }
                for hidden in found {
                    self.pick_up(ground, heap, hidden, who, now);
                }
                if thing.stuff.heavy() {
                    HEAVE_SECS
                } else {
                    match how {
                        How::Plain => LIFT_SECS,
                        How::Gentle => GENTLE_SECS,
                        How::Yank => YANK_SECS,
                    }
                }
            }
            Hand::Peek(who) => {
                self.light -= PEEK_COST;
                let Ok(seen) = self.heaps[heap].peek(item) else {
                    return;
                };
                let first =
                    seen.first()
                        .map(|&hidden| match self.heaps[heap].hidden[hidden].what {
                            Hoard::Find(find) => find.name,
                            Hoard::Map => "A torn map",
                        });
                self.events.push(Event::Peeked {
                    who,
                    stuff: thing.stuff,
                    first,
                    count: seen.len(),
                });
                let peek = Beat::new(Gesture::Peek, ExpressionKind::Curious, PEEK_SECS);
                ground.direct(who, vec![Step::FaceX(face), Step::Beat(peek)], now);
                PEEK_SECS
            }
            Hand::Squeeze(who) => {
                if self.basket.len() >= BASKET {
                    self.events.push(Event::BasketFull);
                    return;
                }
                match self.heaps[heap].squeeze(item) {
                    Err(Refusal::NoGap) | Err(Refusal::Gone) => {
                        self.events.push(Event::NoGap { who });
                        return;
                    }
                    Ok(fetched) => {
                        self.light -= SQUEEZE_COST;
                        self.events.push(Event::Squeezed {
                            who,
                            count: fetched.len(),
                        });
                        // In under the board from the front, and back out again.
                        let back = ground.position(who).unwrap_or(site.stand);
                        let gap = (face, (site.ground + 2) as f32);
                        let crawl = Beat::new(Gesture::Crouch, ExpressionKind::Determined, 0.9);
                        ground.direct(
                            who,
                            vec![
                                Step::Walk { to: gap },
                                Step::Beat(crawl),
                                Step::Walk { to: back },
                                Step::FaceX(face),
                            ],
                            now,
                        );
                        for hidden in fetched {
                            self.heaps[heap].hidden[hidden].place = Place::Open;
                            self.pick_up(ground, heap, hidden, who, now);
                        }
                        SQUEEZE_SECS
                    }
                }
            }
        };
        self.phase = Phase::Working {
            heap,
            until: now + seconds,
        };
    }

    /// Something has come out into the open: a map is kept at once, and a find goes into the
    /// basket if there is room (otherwise it lies there until there is).
    fn pick_up(&mut self, ground: &mut Playground, heap: usize, hidden: usize, who: Id, now: f32) {
        let site = self.sites[heap];
        let what = self.heaps[heap].hidden[hidden].what;
        let at = self.heaps[heap].hidden[hidden].at;
        let lies = self.heaps[heap].hidden[hidden].lies;
        let full = matches!(what, Hoard::Find(_)) && self.basket.len() >= BASKET;
        if full {
            self.events.push(Event::BasketFull);
            return;
        }
        self.heaps[heap].take(hidden);
        self.shown = Some((what, (site.left + at, site.ground - lies), now));
        match what {
            Hoard::Map => {
                self.maps.push(who);
                self.events.push(Event::Map { who });
                for id in self.crew.party.clone() {
                    let mut wonder = Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.0);
                    wonder.cue = Some(Cue::Exclaim);
                    ground.direct(id, vec![Step::Beat(wonder)], now);
                }
            }
            Hoard::Find(find) => {
                self.basket.push(find.id);
                self.events.push(Event::Got { find: find.id });
                if let Some(inspector) = self.crew.first(crew::inspector)
                    && let Some(character) = self.crew.character(inspector)
                {
                    let look = character.flourish(HabitCue::Meal).unwrap_or_else(|| {
                        Beat::new(ActionKind::Eat, ExpressionKind::Curious, 1.0)
                    });
                    ground.direct(inspector, vec![Step::Beat(look)], now);
                    self.events.push(Event::Looked {
                        who: inspector,
                        find: find.id,
                    });
                }
                let finder = self.crew.character(who).cloned();
                if let Some(character) = finder {
                    let beats = delighted(&character, find.tier);
                    let mut steps: Vec<Step> = beats.into_iter().map(Step::Beat).collect();
                    steps.insert(
                        0,
                        Step::Beat(Beat::new(ActionKind::Idle, ExpressionKind::Joy, 0.2)),
                    );
                    ground.direct(who, steps, now);
                }
            }
        }
    }

    /// Picks up something lying in the open at the heap the party is at, once there is room.
    pub fn take(&mut self, ground: &mut Playground, heap: usize, hidden: usize, now: f32) {
        if self.at() != Some(heap) {
            return;
        }
        if self.heaps[heap].hidden.get(hidden).map(|h| h.place) != Some(Place::Open) {
            return;
        }
        let who = self.crew.party.first().copied().unwrap_or_default();
        self.pick_up(ground, heap, hidden, who, now);
    }

    /// Puts something from the basket back, to make room: it is left on the track.
    pub fn put_back(&mut self, slot: usize) {
        if matches!(self.phase, Phase::Leaving { .. } | Phase::Over) || slot >= self.basket.len() {
            return;
        }
        let find = self.basket.remove(slot);
        self.events.push(Event::PutBack { find });
    }

    /// Calls the outing to an end: everyone heads home with what is in the basket.
    pub fn head_home(&mut self, ground: &mut Playground, now: f32) {
        if !matches!(self.phase, Phase::Leaving { .. } | Phase::Over) {
            self.leave(ground, Ending::Chose, now);
        }
    }

    fn leave(&mut self, ground: &mut Playground, why: Ending, now: f32) {
        ground.reserve(self.crew.party.clone());
        for (index, id) in self.crew.party.clone().into_iter().enumerate() {
            let tired = why == Ending::Dusk
                && ground
                    .character(id)
                    .is_some_and(|c| c.kind == TemperamentKind::Lazybones);
            let mut steps = Vec::new();
            if tired {
                steps.push(Step::Beat(Beat::new(
                    Gesture::Yawn,
                    ExpressionKind::Yawning,
                    1.0,
                )));
            }
            let (x, y) = self.way_home;
            steps.push(Step::Walk {
                to: (x - 20.0 * index as f32, y + 4.0 * (index % 2) as f32),
            });
            ground.direct(id, steps, now);
        }
        self.events.push(Event::Leaving(why));
        self.phase = Phase::Leaving { since: now, why };
    }

    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        for motion in &mut self.motion {
            motion.settle(now);
        }
        self.cracks.retain(|(_, since)| now - since < CRACK_SECS);
        if self
            .shown
            .is_some_and(|(_, _, since)| now - since > SHOW_SECS)
        {
            self.shown = None;
        }
        self.update_signs(now);
        if !self.dusk_told && self.light < self.dusk && self.out() {
            self.dusk_told = true;
            self.events.push(Event::Dusk);
        }
        let leader = self.crew.party.first().copied();
        match self.phase {
            Phase::Arriving { since } => {
                let walking = leader.is_some_and(|id| ground.walking(id));
                if !walking || now - since > 6.0 {
                    self.phase = Phase::Choosing;
                }
            }
            Phase::Going { heap } => {
                let walking = leader.is_some_and(|id| ground.walking(id));
                if !walking {
                    self.phase = Phase::At { heap };
                }
            }
            Phase::Working { heap, until } => {
                if now >= until {
                    self.phase = Phase::At { heap };
                }
            }
            Phase::Leaving { since, .. } => {
                let gone = self.crew.party.iter().all(|id| !ground.busy(*id));
                if gone || now - since > 8.0 {
                    self.phase = Phase::Over;
                }
            }
            Phase::Choosing | Phase::At { .. } | Phase::Over => {}
        }
        if self.light <= 0.0 && matches!(self.phase, Phase::Choosing | Phase::At { .. }) {
            self.leave(ground, Ending::Dusk, now);
        }
    }

    /// Whether the party is out on the track, rather than arriving or going home.
    fn out(&self) -> bool {
        matches!(
            self.phase,
            Phase::Choosing | Phase::Going { .. } | Phase::At { .. } | Phase::Working { .. }
        )
    }

    /// Glints come and go where things lie hidden, fainter the deeper they are, and the rarest
    /// only once the light starts to go.
    fn update_signs(&mut self, now: f32) {
        for heap in 0..self.heaps.len() {
            for hidden in 0..self.heaps[heap].hidden.len() {
                let what = &self.heaps[heap].hidden[hidden];
                if !matches!(what.place, Place::Under(_) | Place::Inside(_)) {
                    continue;
                }
                let sign = &mut self.signs[heap][hidden];
                if now < sign.next {
                    continue;
                }
                let tier = match what.what {
                    Hoard::Find(find) => find.tier,
                    Hoard::Map => Tier::Uncommon,
                };
                let (low, high) = match tier {
                    Tier::Common => (3.0, 5.0),
                    Tier::Uncommon => (4.0, 7.0),
                    Tier::Rare | Tier::Exceptional => (6.0, 10.0),
                };
                sign.next = now + self.dice.range(low, high);
                if tier >= Tier::Rare && self.light > self.dusk {
                    continue;
                }
                sign.until = now + SIGN_SECS;
                if let Some(glinted) = self.glinted[heap].get_mut(hidden) {
                    *glinted = true;
                }
            }
        }
    }

    /// The heaps as they stand, each among everyone on its own row.
    pub fn props(&self, now: f32) -> Vec<Prop> {
        self.heaps
            .iter()
            .zip(&self.sites)
            .zip(&self.motion)
            .map(|((heap, site), motion)| {
                stuff::heap_prop(heap, site, motion, now, self.reduce_motion)
            })
            .collect()
    }

    /// The track dimming as the light goes, glints where things lie hidden, what has been seen of
    /// them, the thing under the pointer, cracks, something found held up, and the basket.
    pub fn draw(&self, scene: &mut Canvas, hovered: Option<(usize, usize)>, now: f32) {
        dim(scene, self.light, self.hour_dark);
        for (heap, site) in self.sites.iter().enumerate() {
            let at = &self.heaps[heap];
            for (index, hidden) in at.hidden.iter().enumerate() {
                let (Place::Under(item) | Place::Inside(item)) = hidden.place else {
                    continue;
                };
                let thing = &at.items[item];
                let point = match hidden.place {
                    Place::Under(_) => (site.left + hidden.at, site.ground - thing.y),
                    _ => (
                        site.left + thing.x + thing.w / 2,
                        site.ground - thing.y - thing.h / 2,
                    ),
                };
                if hidden.known {
                    seen(scene, hidden.what, point);
                }
                let sign = self.signs[heap][index];
                if now < sign.until {
                    let t = 1.0 - (sign.until - now) / SIGN_SECS;
                    let depth = at.depth(item) as f32;
                    let strength = (1.0 - depth * 0.18).max(0.4);
                    glint(
                        scene,
                        shine(hidden.what),
                        point,
                        t,
                        strength,
                        self.reduce_motion,
                    );
                }
            }
        }
        if let Some((heap, item)) = hovered
            && let Some(thing) = self.heaps.get(heap).and_then(|at| at.items.get(item))
            && !thing.lifted
        {
            let here = self.at() == Some(heap);
            let color = if here {
                rgb(0xf5d25e)
            } else {
                rgba(0xfdfbf5, 200)
            };
            let (left, top, w, h) = scene_rect(&self.sites[heap], thing);
            corners(scene, (left - 1, top - 1, w + 2, h + 2), color);
        }
        for &((x, y), since) in &self.cracks {
            let t = if self.reduce_motion {
                0.5
            } else {
                ((now - since) / CRACK_SECS).clamp(0.0, 1.0)
            };
            crack(scene, (x, y), t);
        }
        if let Some((what, (x, y), since)) = self.shown {
            let rise = if self.reduce_motion {
                10.0
            } else {
                ((now - since) / 0.4).min(1.0) * 10.0
            };
            let picture = match what {
                Hoard::Find(find) => art::icon(find.id),
                Hoard::Map => map_icon(),
            };
            let (left, top) = (x - 4, y - 12 - rise as i32);
            rect(scene, left - 2, top - 2, 13, 13, rgba(0xf6eed8, 230));
            blit(scene, &picture, left, top);
            let rare =
                matches!(what, Hoard::Find(find) if find.tier >= Tier::Rare) || what == Hoard::Map;
            if rare {
                for (dx, dy) in [(-5, -3), (11, 1), (3, -6)] {
                    put(scene, left + dx, top + dy, rgb(0xfff4c0));
                }
            }
        }
        self.draw_basket(scene);
    }

    /// Where each of the basket's slots is drawn: left, top, and the size of one.
    fn basket_frame(&self, width: u32) -> (i32, i32, i32) {
        const SLOT: i32 = 11;
        let across = SLOT * BASKET as i32 + 3;
        (width as i32 - across - 4, 4, SLOT)
    }

    /// The basket's slot under a point in the scene, for putting something back.
    pub fn basket_slot_at(&self, x: f32, y: f32, width: u32) -> Option<usize> {
        let (left, top, slot) = self.basket_frame(width);
        let (x, y) = (x as i32 - left - 2, y as i32 - top - 2);
        if x < 0 || y < 0 || y >= slot {
            return None;
        }
        let index = (x / slot) as usize;
        (index < self.basket.len()).then_some(index)
    }

    /// The basket in the top right corner, the light left under it, and any maps found beside it.
    fn draw_basket(&self, scene: &mut Canvas) {
        let (left, top, slot) = self.basket_frame(scene.width());
        let width = slot * BASKET as i32 + 3;
        rect(scene, left, top, width, slot + 7, rgba(0x2a2018, 150));
        for index in 0..BASKET as i32 {
            let (x, y) = (left + 2 + index * slot, top + 2);
            rect(scene, x, y, slot - 1, slot - 1, rgba(0xf6eed8, 60));
            if let Some(id) = self.basket.get(index as usize) {
                blit(scene, &art::icon(id), x, y);
            }
        }
        let share = self.light_left();
        let bar = ((width - 4) as f32 * share) as i32;
        let gold = mix(rgb(0x6a5a9a), rgb(0xf5d25e), share);
        scene.fill_rect(left + 2, top + slot + 2, bar, 2, gold);
        for (index, _) in self.maps.iter().enumerate() {
            let x = left - 13 - index as i32 * 11;
            rect(scene, x - 1, top, 11, slot + 1, rgba(0x2a2018, 150));
            blit(scene, &map_icon(), x, top + 2);
        }
    }

    /// Whether something hidden has glinted, or been seen, so a careful hand knows it is there.
    fn noticed(&self, heap: usize, hidden: usize) -> bool {
        let what = &self.heaps[heap].hidden[hidden];
        let glinted = self
            .glinted
            .get(heap)
            .and_then(|glints| glints.get(hidden))
            .copied()
            .unwrap_or(false);
        matches!(what.place, Place::Under(_) | Place::Inside(_)) && (what.known || glinted)
    }

    /// The cheapest way the party can lift something without risking anything: never pulled out
    /// from under anything but gently, and never yanked open if what is in it might break. The
    /// hand, and what it costs.
    fn safe_lift(&self, heap: usize, item: usize) -> Option<(usize, f32)> {
        let at = &self.heaps[heap];
        let thing = &at.items[item];
        let risky = thing.stuff.container()
            && at.hidden_at(item).iter().any(|&hidden| {
                let what = &at.hidden[hidden];
                matches!(what.place, Place::Inside(_)) && (!what.known || what.what.fragile())
            });
        let resting = !at.resting_on(item).is_empty();
        let load = at.above(item).len();
        let mut best: Option<(usize, f32)> = None;
        for (index, hand) in self.hands().iter().enumerate() {
            let Hand::Lift(who) = hand else {
                continue;
            };
            let Some((how, _)) = self.crew.lift(*who, thing.stuff.heavy()) else {
                continue;
            };
            if (resting && how != How::Gentle) || (risky && how == How::Yank) {
                continue;
            }
            let spent = cost(how, thing.stuff.heavy(), load);
            if best.is_none_or(|(_, was)| spent < was) {
                best = Some((index, spent));
            }
        }
        best
    }

    /// The cheapest way the party can lift something at all, pulled out from under whatever is on
    /// it if need be, though never yanked open if what is in it might break.
    fn any_lift(&self, heap: usize, item: usize) -> Option<(usize, f32)> {
        let at = &self.heaps[heap];
        let thing = &at.items[item];
        let risky = thing.stuff.container()
            && at.hidden_at(item).iter().any(|&hidden| {
                let what = &at.hidden[hidden];
                matches!(what.place, Place::Inside(_)) && (!what.known || what.what.fragile())
            });
        let load = at.above(item).len();
        let mut best: Option<(usize, f32)> = None;
        for (index, hand) in self.hands().iter().enumerate() {
            let Hand::Lift(who) = hand else {
                continue;
            };
            let Some((how, _)) = self.crew.lift(*who, thing.stuff.heavy()) else {
                continue;
            };
            if risky && how == How::Yank {
                continue;
            }
            let spent = cost(how, thing.stuff.heavy(), load) + TUMBLE_COST * load as f32;
            if best.is_none_or(|(_, was)| spent < was) {
                best = Some((index, spent));
            }
        }
        best
    }

    /// How much a careful hand wants a find in the basket: the rarer the more, and more if there
    /// isn't one in the basket already.
    fn worth(&self, id: &str) -> u8 {
        finds::find(id).map_or(0, |find| {
            find.tier as u8 * 2 + u8::from(!self.basket.contains(&find.id))
        })
    }

    /// What a careful hand would do next, for the renders and the tests. It watches for glints,
    /// and goes to the heap where the most have shown. There it picks up whatever is lying in the
    /// open (putting back the least thing in a full basket for something better), sends a little
    /// one into a gap where something glinted under a board, and works its way to each thing it
    /// saw glint the cheapest safe way: straight off if nothing is on it, eased out by someone
    /// gentle, and otherwise from the top down. With nothing more seen there it goes where
    /// something has glinted; failing that it peeks at what something rests on, then lifts from
    /// the top down whatever might hide something, and pulls things out from under others only
    /// where no china has gleamed. Home when the light is nearly gone, or the heaps have nothing
    /// more to give.
    pub fn canny(&self) -> Move {
        let hands = self.hands();
        let full = self.basket.len() >= BASKET;
        match self.phase {
            Phase::Choosing => self
                .promising_heap(None)
                .or_else(|| {
                    let nowhere = self.heaps.len();
                    self.elsewhere(nowhere, |heap| self.explore(heap).is_some())
                })
                .filter(|_| !full)
                .map_or(Move::Home, Move::Go),
            Phase::At { heap } => {
                if self.light < 4.0 {
                    return Move::Home;
                }
                let at = &self.heaps[heap];
                let least = self
                    .basket
                    .iter()
                    .enumerate()
                    .map(|(slot, id)| (slot, self.worth(id)))
                    .min_by_key(|(_, worth)| *worth);
                for (index, hidden) in at.hidden.iter().enumerate() {
                    let (Place::Open, Hoard::Find(find)) = (hidden.place, hidden.what) else {
                        continue;
                    };
                    match least {
                        _ if !full => return Move::Take(index),
                        Some((slot, worth)) if self.worth(find.id) > worth => {
                            return Move::PutBack(slot);
                        }
                        _ => {}
                    }
                }
                if let Some(&(_, hand, item)) =
                    self.ways_in(heap).iter().min_by(|a, b| a.0.total_cmp(&b.0))
                {
                    return Move::Act { hand, item };
                }
                if let Some(other) = self.promising_heap(Some(heap)) {
                    return Move::Go(other);
                }
                if let Some(peek) = hands.iter().position(|hand| matches!(hand, Hand::Peek(_))) {
                    let unknown = (0..at.items.len()).find(|&item| {
                        let thing = &at.items[item];
                        !thing.lifted
                            && !thing.peeked
                            && (!at.resting_on(item).is_empty() || at.gap(item))
                    });
                    if let Some(item) = unknown {
                        return Move::Act { hand: peek, item };
                    }
                }
                if full {
                    return Move::Home;
                }
                // From the top down, whatever might hide something; then anywhere else that can
                // be, nearest first; then pulled out from under, where no china has gleamed.
                if let Some((_, hand, item)) = self.explore(heap) {
                    return Move::Act { hand, item };
                }
                if let Some(next) = self.elsewhere(heap, |heap| self.explore(heap).is_some()) {
                    return Move::Go(next);
                }
                if let Some((_, hand, item)) = self.pull(heap) {
                    return Move::Act { hand, item };
                }
                self.elsewhere(heap, |heap| self.pull(heap).is_some())
                    .map_or(Move::Home, Move::Go)
            }
            _ => Move::Wait,
        }
    }

    /// The cheapest thing to lift safely from the top of a heap that might hide something, or
    /// keeps something down.
    fn explore(&self, heap: usize) -> Option<(f32, usize, usize)> {
        let at = &self.heaps[heap];
        (0..at.items.len())
            .filter(|&item| {
                let thing = &at.items[item];
                let empty = thing.peeked && at.hidden_at(item).is_empty();
                let keeps_down = at
                    .items
                    .iter()
                    .any(|other| !other.lifted && other.y < thing.y && other.overlaps(thing));
                !thing.lifted && at.resting_on(item).is_empty() && (!empty || keeps_down)
            })
            .filter_map(|item| {
                self.safe_lift(heap, item)
                    .map(|(hand, spent)| (spent, hand, item))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
    }

    /// The cheapest thing to pull out from under others in a heap, where no china has gleamed
    /// in it or above it, if there is light enough for it.
    fn pull(&self, heap: usize) -> Option<(f32, usize, usize)> {
        let at = &self.heaps[heap];
        let gleamed = |item: usize| {
            (0..at.hidden.len()).any(|hidden| {
                matches!(at.hidden[hidden].place, Place::Under(i) | Place::Inside(i) if i == item)
                    && self.noticed(heap, hidden)
                    && at.hidden[hidden].what.fragile()
            })
        };
        (0..at.items.len())
            .filter(|&item| !at.items[item].lifted && !at.items[item].peeked)
            .filter(|&item| !gleamed(item) && !at.above(item).into_iter().any(gleamed))
            .filter_map(|item| {
                self.any_lift(heap, item)
                    .map(|(hand, spent)| (spent, hand, item))
            })
            .filter(|(spent, _, _)| *spent < self.light)
            .min_by(|a, b| a.0.total_cmp(&b.0))
    }

    /// The nearest heap other than `not` where `worth` holds.
    fn elsewhere(&self, not: usize, worth: impl Fn(usize) -> bool) -> Option<usize> {
        (0..self.heaps.len())
            .filter(|&heap| heap != not && worth(heap))
            .min_by(|&a, &b| {
                distance(self.from, self.sites[a].stand)
                    .total_cmp(&distance(self.from, self.sites[b].stand))
            })
    }

    /// Every safe way to get at something that has glinted or been seen in a heap: a little one
    /// into the gap over it, the thing it is under or in if that can be lifted safely, or else
    /// the top of what is on it. Each with what it costs, the hand, and what to do it to.
    fn ways_in(&self, heap: usize) -> Vec<(f32, usize, usize)> {
        let at = &self.heaps[heap];
        let mut targets: Vec<usize> = (0..at.hidden.len())
            .filter(|&hidden| self.noticed(heap, hidden))
            .filter_map(|hidden| match at.hidden[hidden].place {
                Place::Under(item) | Place::Inside(item) => Some(item),
                _ => None,
            })
            .collect();
        targets.sort_unstable();
        targets.dedup();
        let squeeze = self
            .hands()
            .iter()
            .position(|hand| matches!(hand, Hand::Squeeze(_)))
            .filter(|_| self.basket.len() < BASKET);
        let mut ways = Vec::new();
        for target in targets {
            let under = (0..at.hidden.len()).any(|hidden| {
                at.hidden[hidden].place == Place::Under(target) && self.noticed(heap, hidden)
            });
            if let Some(hand) = squeeze
                && under
                && at.gap(target)
            {
                ways.push((SQUEEZE_COST, hand, target));
                continue;
            }
            if let Some((hand, spent)) = self.safe_lift(heap, target) {
                ways.push((spent, hand, target));
                continue;
            }
            for top in at.above(target) {
                if at.resting_on(top).is_empty()
                    && let Some((hand, spent)) = self.safe_lift(heap, top)
                {
                    ways.push((spent + LIFT_COST, hand, top));
                }
            }
        }
        ways
    }

    /// The heap other than `not` with the most that has glinted or been seen and can be got at
    /// safely, the nearer first.
    fn promising_heap(&self, not: Option<usize>) -> Option<usize> {
        (0..self.heaps.len())
            .filter(|&heap| Some(heap) != not)
            .map(|heap| (heap, self.ways_in(heap).len()))
            .filter(|(_, noticed)| *noticed > 0)
            .max_by(|a, b| {
                a.1.cmp(&b.1).then_with(|| {
                    distance(self.from, self.sites[b.0].stand)
                        .total_cmp(&distance(self.from, self.sites[a.0].stand))
                })
            })
            .map(|(heap, _)| heap)
    }
}

/// Four little corners round something: the thing the pointer is on.
fn corners(scene: &mut Canvas, (left, top, w, h): (i32, i32, i32, i32), color: Rgba) {
    let (right, bottom) = (left + w - 1, top + h - 1);
    for (x, y, dx, dy) in [
        (left, top, 1, 1),
        (right, top, -1, 1),
        (left, bottom, 1, -1),
        (right, bottom, -1, -1),
    ] {
        put(scene, x, y, color);
        put(scene, x + dx, y, color);
        put(scene, x, y + dy, color);
    }
}

/// Something seen where it lies hidden: its icon, pale, on a dotted card, so it reads as seen
/// through the gaps rather than out in the open.
fn seen(scene: &mut Canvas, what: Hoard, (x, y): (i32, i32)) {
    let picture = match what {
        Hoard::Find(find) => art::icon(find.id),
        Hoard::Map => map_icon(),
    };
    let (left, top) = (x - 4, y - 10);
    for step in 0..12 {
        if step % 2 == 0 {
            put(scene, left - 2 + step, top - 2, rgba(0xf6eed8, 200));
            put(scene, left - 2 + step, top + 10, rgba(0xf6eed8, 200));
            put(scene, left - 2, top - 2 + step, rgba(0xf6eed8, 200));
            put(scene, left + 10, top - 2 + step, rgba(0xf6eed8, 200));
        }
    }
    rect(scene, left - 1, top - 1, 11, 11, rgba(0x2a2018, 90));
    for py in 0..picture.height() as i32 {
        for px in 0..picture.width() as i32 {
            let pixel = picture.get(px, py);
            if pixel.a > 0 {
                put(
                    scene,
                    left + px,
                    top + py,
                    Rgba::new(pixel.r, pixel.g, pixel.b, 170),
                );
            }
        }
    }
}

/// A glint where something lies hidden, `t` of the way through showing: metal warm, china and
/// glass cool, paper and straw a corner showing. With reduced motion, a still mark that comes and
/// goes.
fn glint(scene: &mut Canvas, shine: Shine, (x, y): (i32, i32), t: f32, strength: f32, still: bool) {
    let bright = if still {
        strength
    } else {
        (t * std::f32::consts::PI).sin() * strength
    };
    let alpha = (255.0 * bright) as u8;
    match shine {
        Shine::Glint | Shine::Gleam => {
            let color = if shine == Shine::Glint {
                0xfff0b0
            } else {
                0xe4f4ff
            };
            put(scene, x, y, rgba(color, alpha));
            let reach = if bright > 0.6 { 2 } else { 1 };
            for step in 1..=reach {
                let fade = rgba(color, (f32::from(alpha) * 0.7) as u8);
                for (dx, dy) in [(step, 0), (-step, 0), (0, step), (0, -step)] {
                    put(scene, x + dx, y + dy, fade);
                }
            }
        }
        Shine::Corner(color) => {
            for (dx, dy) in [(0, 0), (1, 0), (0, -1), (2, 0)] {
                put(scene, x + dx, y + dy, rgba(color, alpha));
            }
            put(scene, x + 1, y - 1, rgba(0x5a4a3a, alpha / 2));
        }
    }
}

/// A crack where something broke, `t` of the way through showing: a white zigzag and chips.
fn crack(scene: &mut Canvas, (x, y): (i32, i32), t: f32) {
    let alpha = (255.0 * (1.0 - t * t)) as u8;
    let white = rgba(0xffffff, alpha);
    for (dx, dy) in [(-3, -2), (-2, -1), (-1, -1), (0, 0), (1, 0), (2, 1), (3, 1)] {
        put(scene, x + dx, y + dy, white);
    }
    for (dx, dy) in [(-2, -4), (2, -3), (4, -1)] {
        put(scene, x + dx, y + dy, rgba(0xe4f4ff, alpha));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::station::SCENE_WIDTH;
    use crate::track;

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    fn named(cast: &Cast, name: &str) -> Id {
        cast.members
            .iter()
            .find(|member| member.name == name)
            .unwrap_or_else(|| panic!("no {name} in the sample"))
            .id
    }

    fn outing(cast: &Cast, names: &[&str], close_pair: bool, seed: u64) -> (Playground, Scavenge) {
        let party: Vec<Id> = names.iter().map(|name| named(cast, name)).collect();
        let mut ground = track::open(cast, &party, 0.0, &Default::default());
        let outset = Outset {
            party,
            drought: 0,
            close_pair,
            influence: Influence::default(),
            maps_held: 0,
            seed,
        };
        let scavenge = Scavenge::new(&mut ground, outset, |_| false, 0.0);
        (ground, scavenge)
    }

    fn run(
        ground: &mut Playground,
        scavenge: &mut Scavenge,
        cast: &Cast,
        from: f32,
        to: f32,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            scavenge.tick(ground, now);
            events.extend(scavenge.take_events());
        }
        events
    }

    /// Takes the party to a heap and lets it settle there.
    fn go_to(
        ground: &mut Playground,
        scavenge: &mut Scavenge,
        cast: &Cast,
        heap: usize,
        now: f32,
    ) -> f32 {
        scavenge.choose(ground, heap, now);
        let mut later = now;
        while scavenge.phase() != (Phase::At { heap }) && later < now + 30.0 {
            later += 1.0 / 30.0;
            ground.tick(cast, later);
            scavenge.tick(ground, later);
        }
        assert_eq!(scavenge.phase(), Phase::At { heap });
        later
    }

    /// Lays a heap out by hand at a site: two stones, a plank across them, and a sack on it, with
    /// whatever is wanted hidden under the plank.
    fn bridge(scavenge: &mut Scavenge, heap: usize, under: Option<&str>) {
        let mut built = Heap::build(
            &heap::Recipe {
                ground: (&[Stuff::Stone], (2, 2)),
                across: (&[Stuff::Plank], (1, 1)),
                top: (&[Stuff::Sack], (1, 1)),
            },
            60,
            &mut Dice::new(1),
        );
        built.hidden.clear();
        if let Some(id) = under {
            let plank = built
                .items
                .iter()
                .position(|item| item.stuff == Stuff::Plank)
                .unwrap();
            let sack = built
                .items
                .iter()
                .position(|item| item.stuff == Stuff::Sack)
                .unwrap();
            built.hidden.push(heap::Hidden {
                what: Hoard::Find(finds::find(id).unwrap()),
                place: Place::Under(plank),
                cracked: false,
                known: false,
                at: built.items[sack].x + built.items[sack].w / 2,
                lies: 0,
            });
        }
        scavenge.signs[heap] = vec![Sign::default(); built.hidden.len()];
        scavenge.heaps[heap] = built;
    }

    fn index_of(scavenge: &Scavenge, heap: usize, stuff: Stuff) -> usize {
        scavenge.heaps[heap]
            .items
            .iter()
            .position(|item| item.stuff == stuff && !item.lifted)
            .unwrap()
    }

    fn hand_of(scavenge: &Scavenge, wanted: impl Fn(&Hand) -> bool) -> usize {
        scavenge.hands().iter().position(wanted).unwrap()
    }

    #[test]
    fn from_the_top_down_the_teacup_comes_home_whole() {
        let cast = sample();
        let (mut ground, mut scavenge) = outing(&cast, &["Fig"], false, 1);
        run(&mut ground, &mut scavenge, &cast, 0.0, 6.0);
        bridge(&mut scavenge, 0, Some("china_teacup"));
        let mut now = go_to(&mut ground, &mut scavenge, &cast, 0, 6.0);
        let sack = index_of(&scavenge, 0, Stuff::Sack);
        scavenge.act(&mut ground, sack, now);
        run(&mut ground, &mut scavenge, &cast, now, now + 2.0);
        now += 2.0;
        let plank = index_of(&scavenge, 0, Stuff::Plank);
        scavenge.act(&mut ground, plank, now);
        assert!(!scavenge.take_events().contains(&Event::Cracked));
        assert_eq!(scavenge.basket(), &["china_teacup"]);
    }

    #[test]
    fn pulling_the_plank_from_under_the_sack_buries_the_teacup_and_cracks_it_but_the_basket_is_safe()
     {
        let cast = sample();
        let (mut ground, mut scavenge) = outing(&cast, &["Fig"], false, 2);
        run(&mut ground, &mut scavenge, &cast, 0.0, 6.0);
        bridge(&mut scavenge, 0, Some("china_teacup"));
        scavenge.basket = vec!["china_teacup", "tin_soldier"];
        let mut now = go_to(&mut ground, &mut scavenge, &cast, 0, 6.0);
        let plank = index_of(&scavenge, 0, Stuff::Plank);
        let light = scavenge.light;
        scavenge.act(&mut ground, plank, now);
        let events = scavenge.take_events();
        assert!(events.contains(&Event::Tumbled { count: 1 }), "{events:?}");
        assert!(events.contains(&Event::Cracked));
        assert!(scavenge.light < light);
        assert_eq!(
            scavenge.basket(),
            &["china_teacup", "tin_soldier"],
            "nothing in the basket is touched"
        );
        // It is under the sack now, cracked; lifting the sack brings it out.
        now += 2.0;
        run(&mut ground, &mut scavenge, &cast, now - 2.0, now);
        let sack = index_of(&scavenge, 0, Stuff::Sack);
        scavenge.act(&mut ground, sack, now);
        assert_eq!(
            scavenge.basket(),
            &["china_teacup", "tin_soldier", "cracked_teacup"]
        );
    }

    #[test]
    fn a_patient_one_eases_the_plank_out_and_nothing_cracks() {
        let cast = sample();
        let (mut ground, mut scavenge) = outing(&cast, &["Mochi"], false, 3);
        run(&mut ground, &mut scavenge, &cast, 0.0, 6.0);
        bridge(&mut scavenge, 0, Some("china_teacup"));
        let now = go_to(&mut ground, &mut scavenge, &cast, 0, 6.0);
        let plank = index_of(&scavenge, 0, Stuff::Plank);
        let light = scavenge.light;
        scavenge.act(&mut ground, plank, now);
        let events = scavenge.take_events();
        assert!(!events.contains(&Event::Cracked), "{events:?}");
        assert_eq!(scavenge.basket(), &["china_teacup"]);
        assert!(
            light - scavenge.light > LIFT_COST,
            "easing it out takes longer, and more light"
        );
    }

    #[test]
    fn an_impulsive_one_yanks_cheaply_and_what_breaks_inside_cracks() {
        let cast = sample();
        let (mut ground, mut scavenge) = outing(&cast, &["Biscuit"], false, 4);
        run(&mut ground, &mut scavenge, &cast, 0.0, 6.0);
        bridge(&mut scavenge, 0, None);
        let sack = index_of(&scavenge, 0, Stuff::Sack);
        scavenge.heaps[0].hidden.push(heap::Hidden {
            what: Hoard::Find(finds::find("china_teacup").unwrap()),
            place: Place::Inside(sack),
            cracked: false,
            known: true,
            at: 10,
            lies: 0,
        });
        scavenge.signs[0].push(Sign::default());
        let now = go_to(&mut ground, &mut scavenge, &cast, 0, 6.0);
        let light = scavenge.light;
        scavenge.act(&mut ground, sack, now);
        assert!(
            (light - scavenge.light - YANK_COST).abs() < 0.01,
            "a yank is quick"
        );
        assert_eq!(scavenge.basket(), &["cracked_teacup"]);
    }

    #[test]
    fn heavy_things_need_someone_strong_or_two_together() {
        let cast = sample();
        let beam_heap = |scavenge: &mut Scavenge| {
            let mut built = Heap::build(
                &heap::Recipe {
                    ground: (&[Stuff::Stone], (2, 2)),
                    across: (&[Stuff::Beam], (1, 1)),
                    top: (&[Stuff::Tin], (0, 0)),
                },
                60,
                &mut Dice::new(5),
            );
            built.hidden.clear();
            scavenge.heaps[0] = built;
            scavenge.signs[0].clear();
        };
        for (names, lifts) in [
            (&["Mochi"][..], false),
            (&["Biscuit"][..], true),
            (&["Mochi", "Fig"][..], true),
        ] {
            let (mut ground, mut scavenge) = outing(&cast, names, false, 5);
            run(&mut ground, &mut scavenge, &cast, 0.0, 6.0);
            beam_heap(&mut scavenge);
            let now = go_to(&mut ground, &mut scavenge, &cast, 0, 6.0);
            let beam = index_of(&scavenge, 0, Stuff::Beam);
            let light = scavenge.light;
            scavenge.act(&mut ground, beam, now);
            let events = scavenge.take_events();
            assert_eq!(
                scavenge.heaps[0].items[beam].lifted, lifts,
                "{names:?}: {events:?}"
            );
            if !lifts {
                assert!(events.iter().any(|e| matches!(e, Event::TooHeavy { .. })));
                assert_eq!(scavenge.light, light, "trying costs nothing");
            }
            if names.len() == 2 {
                assert!(events.iter().any(|e| matches!(e, Event::Together { .. })));
            }
        }
    }

    #[test]
    fn a_curious_one_peeks_and_a_little_one_squeezes_in() {
        let cast = sample();
        let (mut ground, mut scavenge) = outing(&cast, &["Tansy"], false, 6);
        run(&mut ground, &mut scavenge, &cast, 0.0, 6.0);
        bridge(&mut scavenge, 0, Some("tin_soldier"));
        let now = go_to(&mut ground, &mut scavenge, &cast, 0, 6.0);
        let plank = index_of(&scavenge, 0, Stuff::Plank);
        scavenge.set_hand(hand_of(&scavenge, |hand| matches!(hand, Hand::Peek(_))));
        scavenge.act(&mut ground, plank, now);
        let events = scavenge.take_events();
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Peeked {
                first: Some("A tin soldier"),
                count: 1,
                ..
            }
        )));
        assert!(scavenge.heaps[0].hidden[0].known);
        assert!(
            !scavenge.heaps[0].items[plank].lifted,
            "a peek moves nothing"
        );
        assert!(scavenge.describe(0, plank).contains("tin soldier"));

        let (mut ground, mut scavenge) = outing(&cast, &["Mochi", "Pip"], false, 7);
        run(&mut ground, &mut scavenge, &cast, 0.0, 6.0);
        bridge(&mut scavenge, 0, Some("china_teacup"));
        let now = go_to(&mut ground, &mut scavenge, &cast, 0, 6.0);
        let plank = index_of(&scavenge, 0, Stuff::Plank);
        let before = scavenge.heaps[0].items.clone();
        scavenge.set_hand(hand_of(&scavenge, |hand| matches!(hand, Hand::Squeeze(_))));
        scavenge.act(&mut ground, plank, now);
        assert_eq!(scavenge.basket(), &["china_teacup"], "brought out whole");
        assert_eq!(scavenge.heaps[0].items, before, "and nothing moved");
        // Not under a stone on the ground: there is no gap.
        let stone = index_of(&scavenge, 0, Stuff::Stone);
        scavenge.phase = Phase::At { heap: 0 };
        scavenge.act(&mut ground, stone, now + 3.0);
        assert!(
            scavenge
                .take_events()
                .iter()
                .any(|e| matches!(e, Event::NoGap { .. }))
        );
    }

    #[test]
    fn one_that_looks_things_over_knows_whats_inside_and_looks_over_what_it_finds() {
        let cast = sample();
        for seed in 0..30 {
            let (_, scavenge) = outing(&cast, &["Biscuit"], false, seed);
            for heap in &scavenge.heaps {
                for hidden in &heap.hidden {
                    if matches!(hidden.place, Place::Inside(_)) {
                        assert!(hidden.known);
                    }
                }
            }
            let (_, plain) = outing(&cast, &["Fig"], false, seed);
            assert!(
                plain
                    .heaps
                    .iter()
                    .all(|heap| heap.hidden.iter().all(|h| !h.known))
            );
        }
        let (mut ground, mut scavenge) = outing(&cast, &["Fig", "Biscuit"], false, 8);
        run(&mut ground, &mut scavenge, &cast, 0.0, 6.0);
        bridge(&mut scavenge, 0, Some("tin_soldier"));
        let now = go_to(&mut ground, &mut scavenge, &cast, 0, 6.0);
        let sack = index_of(&scavenge, 0, Stuff::Sack);
        scavenge.act(&mut ground, sack, now);
        let plank = index_of(&scavenge, 0, Stuff::Plank);
        scavenge.phase = Phase::At { heap: 0 };
        scavenge.act(&mut ground, plank, now + 2.0);
        assert!(scavenge.take_events().iter().any(|e| matches!(
            e,
            Event::Looked {
                find: "tin_soldier",
                ..
            }
        )));
    }

    #[test]
    fn the_basket_holds_six_and_whatever_else_waits_in_the_open() {
        let cast = sample();
        let (mut ground, mut scavenge) = outing(&cast, &["Fig"], false, 9);
        run(&mut ground, &mut scavenge, &cast, 0.0, 6.0);
        bridge(&mut scavenge, 0, Some("tin_soldier"));
        scavenge.basket = vec!["straw_hat"; BASKET];
        let now = go_to(&mut ground, &mut scavenge, &cast, 0, 6.0);
        let sack = index_of(&scavenge, 0, Stuff::Sack);
        scavenge.act(&mut ground, sack, now);
        let plank = index_of(&scavenge, 0, Stuff::Plank);
        scavenge.phase = Phase::At { heap: 0 };
        scavenge.act(&mut ground, plank, now + 2.0);
        assert!(scavenge.take_events().contains(&Event::BasketFull));
        assert_eq!(scavenge.basket().len(), BASKET);
        assert_eq!(scavenge.heaps[0].hidden[0].place, Place::Open, "it waits");
        let slot = scavenge.basket_slot_at(SCENE_WIDTH as f32 - 10.0, 9.0, SCENE_WIDTH);
        assert_eq!(slot, Some(BASKET - 1));
        scavenge.put_back(0);
        scavenge.take(&mut ground, 0, 0, now + 3.0);
        assert_eq!(scavenge.basket().last(), Some(&"tin_soldier"));
    }

    #[test]
    fn walking_and_lifting_spend_the_light_and_the_outing_ends_at_dusk() {
        let cast = sample();
        let (mut ground, mut scavenge) = outing(&cast, &["Fig"], false, 10);
        run(&mut ground, &mut scavenge, &cast, 0.0, 6.0);
        let before = scavenge.light;
        let now = go_to(&mut ground, &mut scavenge, &cast, 2, 6.0);
        assert!(scavenge.light < before, "the walk cost nothing");
        // Waiting about costs nothing; lifting does.
        let light = scavenge.light;
        run(&mut ground, &mut scavenge, &cast, now, now + 5.0);
        assert_eq!(scavenge.light, light);
        scavenge.light = 0.5;
        let top = (0..scavenge.heaps[2].items.len())
            .find(|&item| {
                scavenge.heaps[2].resting_on(item).is_empty()
                    && !scavenge.heaps[2].items[item].stuff.heavy()
            })
            .unwrap();
        scavenge.act(&mut ground, top, now + 5.0);
        let events = run(&mut ground, &mut scavenge, &cast, now + 5.0, now + 20.0);
        assert!(events.contains(&Event::Dusk), "{events:?}");
        assert!(events.contains(&Event::Leaving(Ending::Dusk)), "{events:?}");
        assert_eq!(scavenge.phase(), Phase::Over);
    }

    /// Plays a whole scavenge: `careful` as `canny` would, otherwise clicking at whatever is in
    /// the heap at random and wandering from heap to heap. Says what came home, how many things
    /// cracked, and how many maps turned up.
    fn play(cast: &Cast, names: &[&str], seed: u64, careful: bool) -> (Vec<&'static str>, usize) {
        let (mut ground, mut scavenge) = outing(cast, names, false, seed);
        let mut dice = Dice::new(seed ^ 0xca5e);
        let (mut now, mut cracks) = (0.0, 0);
        while scavenge.phase() != Phase::Over && now < 900.0 {
            now += 0.1;
            ground.tick(cast, now);
            scavenge.tick(&mut ground, now);
            cracks += scavenge
                .take_events()
                .iter()
                .filter(|event| **event == Event::Cracked)
                .count();
            if careful {
                match scavenge.canny() {
                    Move::Go(heap) => scavenge.choose(&mut ground, heap, now),
                    Move::Act { hand, item } => {
                        scavenge.set_hand(hand);
                        scavenge.act(&mut ground, item, now);
                    }
                    Move::Take(hidden) => {
                        let heap = scavenge.at().unwrap();
                        scavenge.take(&mut ground, heap, hidden, now);
                    }
                    Move::PutBack(slot) => scavenge.put_back(slot),
                    Move::Home => scavenge.head_home(&mut ground, now),
                    Move::Wait => {}
                }
                continue;
            }
            match scavenge.phase() {
                Phase::Choosing => {
                    let heap = (dice.unit() * 3.0) as usize % 3;
                    scavenge.choose(&mut ground, heap, now);
                }
                Phase::At { heap } => {
                    let open = scavenge.heaps[heap]
                        .hidden
                        .iter()
                        .position(|hidden| hidden.place == Place::Open);
                    let left: Vec<usize> = (0..scavenge.heaps[heap].items.len())
                        .filter(|&item| !scavenge.heaps[heap].items[item].lifted)
                        .collect();
                    if scavenge.basket().len() >= BASKET {
                        scavenge.head_home(&mut ground, now);
                    } else if let Some(hidden) = open {
                        scavenge.take(&mut ground, heap, hidden, now);
                    } else if left.is_empty() || dice.chance(0.08) {
                        let other = (heap + 1 + (dice.unit() * 2.0) as usize % 2) % 3;
                        scavenge.choose(&mut ground, other, now);
                    } else {
                        let item = left[(dice.unit() * left.len() as f32) as usize % left.len()];
                        scavenge.set_hand(0);
                        scavenge.act(&mut ground, item, now);
                    }
                }
                _ => {}
            }
        }
        assert_eq!(scavenge.phase(), Phase::Over, "seed {seed} never went home");
        (scavenge.basket().to_vec(), cracks)
    }

    #[test]
    fn a_careful_hand_brings_home_more_than_a_careless_one_and_cracks_nothing() {
        let cast = sample();
        let value = |basket: &[&str]| {
            basket
                .iter()
                .filter_map(|id| finds::find(id))
                .map(|find| find.tier as u32 * 3 + 1)
                .sum::<u32>()
        };
        let (mut careful_value, mut careless_value, mut careless_cracks) = (0, 0, 0);
        for seed in 0..16 {
            for party in [&["Fig"][..], &["Mochi", "Pip"][..], &["Biscuit"][..]] {
                let (careful, cracks) = play(&cast, party, seed, true);
                assert_eq!(cracks, 0, "a careful hand cracked something, seed {seed}");
                careful_value += value(&careful);
                let (careless, cracks) = play(&cast, party, seed, false);
                careless_value += value(&careless);
                careless_cracks += cracks;
            }
        }
        assert!(
            careless_cracks > 0,
            "pulling things out from under never cracked anything"
        );
        assert!(
            careful_value > careless_value,
            "careful {careful_value}, careless {careless_value}"
        );
    }

    #[test]
    fn with_reduced_motion_lifting_and_tumbling_are_cuts() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (mut ground, mut scavenge) = outing(&cast, &["Fig"], false, 11);
        assert!(scavenge.reduce_motion);
        run(&mut ground, &mut scavenge, &cast, 0.0, 1.0);
        bridge(&mut scavenge, 0, None);
        let now = go_to(&mut ground, &mut scavenge, &cast, 0, 1.0);
        let still = scavenge.props(now).remove(0).sprite;
        let plank = index_of(&scavenge, 0, Stuff::Plank);
        scavenge.act(&mut ground, plank, now);
        // The sack came down: a moment later it is drawn where it landed, never in between.
        let props = scavenge.props(now + stuff::FALL_SECS * 0.5).remove(0);
        let mut settled = scavenge.motion[0].clone();
        settled.falling.clear();
        settled.lifting.clear();
        let landed = stuff::heap_prop(&scavenge.heaps[0], &scavenge.sites[0], &settled, now, true);
        assert_eq!(props.sprite, landed.sprite);
        assert_ne!(props.sprite, still);
    }
}
