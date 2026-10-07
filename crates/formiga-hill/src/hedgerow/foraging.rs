//! A foray along the hedgerow: one outing with a companion or two, until the light goes or the
//! person takes the basket home.
//!
//! The person plays. Everything growing along the hedge and the bank ripens through the outing on
//! its own plant's rhythm: unripe, then turning, then ripe for a while, then over, until it drops.
//! The outing's time is its light, which goes by the second, with every step walked and with every
//! picking, so ripeness is read off each thing's look (see `produce`) and a way round is worked
//! out that arrives at each as it turns. Picking something ripe keeps it. Picking too early just
//! leaves it on the bush, a picking wasted; picking it over loses it, as it squashes or drops, but
//! never anything already in the basket. Rarer things are ripe only briefly and later in the day,
//! and the four-leaf clover only in the last of the light. The basket holds six, so what is worth
//! a place is the person's choice; anything can be put back to make room, and a full basket can go
//! home whenever.
//!
//! Who came along matters, read from who each is. One that looks food over inspects everything
//! before it is picked, so nothing is picked too early, though now and then a ripe one gets
//! eaten. A curious one spots what is hiding under the leaves, and says when something rare is
//! nearly ripe. A little one gets its paws into the low, tucked-in places; a bold one climbs for
//! the high branches, and a close pair can bend a high branch down between them. A patient one
//! picks so gently that ripeness lasts a little longer in its paws. A sweetheart leaves what is
//! left for the birds, and the birds remember it. None of this rules anything out: whatever turns
//! up in the tucked and high places grows within anyone's reach too.

use super::produce::{self, Stage};
use super::{PATCHES, Plant, Reach, Slot};
use crate::actor::Step;
use crate::cast::Id;
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::finds::{self, DROUGHT, FORAGED, Find, NOVELTY, Tier, drawn_to};
use crate::paint::{blit, mix, put, rect, rgb, rgba};
use crate::playground::{Playground, Prop, distance};
use crate::woods::Influence;
use crate::woods::rummage::{delighted, dim};
use formiga_art::{Canvas, ExpressionKind, Rgba};
use formiga_core::{ActionKind, Gesture, Habit, TemperamentKind};

/// The light an outing starts with, before the Hilltop lends any.
pub const LIGHT: f32 = 100.0;
/// How many things the basket holds.
pub const BASKET: usize = 6;
/// How much light goes each second, each pixel walked by a middling companion, each picking, and
/// each climb into the high branches.
const LIGHT_PER_SEC: f32 = 0.4;
const WALK_COST: f32 = 0.035;
const PICK_COST: f32 = 1.5;
const CLIMB_COST: f32 = 2.5;
/// How long before it is ripe a thing starts to turn, and how long it hangs on over before it
/// drops, as shares of the outing.
const TURNING: f32 = 0.08;
const LINGER: f32 = 0.08;
/// From this share of the outing gone, the light is going.
const DUSK: f32 = 0.75;
/// How much longer than its window ripeness lasts in the gentlest paws, as a share of it.
const GENTLE: f32 = 0.5;
/// How likely something growing in an open place is to bear, by its weight: the rarer and the
/// less drawn to it the party, the likelier the place is bare this time.
const BEARING: f32 = 6.0;
/// How long a picking takes, and a climb into the high branches.
const PICK_SECS: f32 = 0.6;
const CLIMB_SECS: f32 = 1.6;
/// How high a climber goes.
const LIFT: f32 = 22.0;
/// How long something picked is held up to look at.
const SHOW_SECS: f32 = 1.4;
/// How long something takes to drop, and lies where it fell.
const FALL_SECS: f32 = 0.45;
const FALLEN_SECS: f32 = 1.6;
/// How long a robin sits by what was left for it before it takes it.
const ROBIN_SECS: f32 = 1.2;
/// How many things left for the birds before they bring something back.
const THANKS: u32 = 3;
/// How far beside the one picking a second companion stands.
const BESIDE: f32 = 22.0;
/// How quickly someone curious spots something hidden, per second, at its most curious.
const SPOTTING: f32 = 0.3;
/// How likely one that looks food over is to eat a ripe common thing there is one of already.
const NIBBLE: f32 = 0.35;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// The companions are walking down the lane.
    Arriving {
        since: f32,
    },
    /// In the lane, deciding where to go.
    Choosing,
    Going {
        patch: usize,
    },
    /// At a patch, picking whatever is ripe.
    At {
        patch: usize,
    },
    Leaving {
        since: f32,
        why: Ending,
    },
    /// Everyone has gone home; the basket is ready to be kept.
    Over,
}

/// What to do next, as a canny forager would.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Pick(usize),
    PutBack(usize),
    Go(usize),
    Wait,
}

/// Why a foray ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    Dusk,
    Chose,
}

/// Something that happened, for the person to be told.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// Someone in the party can get at places only some company can: tucked in, or high up.
    Opened {
        who: Id,
        reach: Reach,
        pair: bool,
    },
    /// Someone curious spotted something hiding.
    Spotted {
        who: Id,
        item: usize,
    },
    /// Looking closely at a patch turned up something hiding there.
    Uncovered {
        item: usize,
    },
    /// Someone curious says something rare is nearly ripe.
    Ripening {
        who: Id,
        item: usize,
    },
    Picked {
        find: &'static str,
        item: usize,
    },
    /// Just over, but picked so gently it came whole.
    Saved {
        who: Id,
        item: usize,
    },
    /// Too early: left on the bush.
    NotYet {
        item: usize,
    },
    /// One that looks food over looked it over, and left it: not yet.
    LookedOver {
        who: Id,
        item: usize,
    },
    /// Picked over: it squashed or dropped.
    Spoilt {
        item: usize,
    },
    /// One that looks food over ate it.
    Ate {
        who: Id,
        item: usize,
    },
    BasketFull,
    PutBack {
        find: &'static str,
    },
    /// Something nobody picked went over and dropped.
    Dropped {
        item: usize,
    },
    /// A sweetheart left what was ripe for the birds.
    ForTheBirds {
        who: Id,
        count: u32,
    },
    /// And the birds brought something back.
    Thanked {
        who: Id,
        item: usize,
    },
    /// The light is going.
    Dusk,
    Leaving(Ending),
}

/// What a foray sets out with. The same shape as every Woods outing's, so an expedition could one
/// day hand one on to the next.
pub struct Outset {
    /// Who came along; the first leads.
    pub party: Vec<Id>,
    /// Forays in a row that brought nothing new home.
    pub drought: u32,
    /// Whether the two who came are close friends.
    pub close_pair: bool,
    /// What the Hilltop lends the Woods: light, and the rare things sooner.
    pub influence: Influence,
    pub seed: u64,
}

/// What became of something growing.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Fate {
    Growing,
    /// Off the bush: picked, eaten.
    Taken,
    /// Over, and dropped then.
    Fell {
        since: f32,
    },
    /// Left for the birds: a robin takes it then.
    ForBirds {
        at: f32,
    },
    Gone,
}

/// One thing growing on this foray.
#[derive(Clone, Debug)]
struct Item {
    slot: Slot,
    patch: usize,
    find: &'static Find,
    /// When it is ripe, and for how long, as shares of the outing gone.
    ripe_at: f32,
    window: f32,
    /// Seen: not hidden, or found.
    revealed: bool,
    /// Whether anyone has said it is nearly ripe.
    told: bool,
    fate: Fate,
    /// Where it was put down by a robin, rather than growing: drawn lying there.
    gift: bool,
}

impl Item {
    fn stage(&self, gone: f32) -> Stage {
        let t = gone - self.ripe_at;
        if t < -TURNING {
            Stage::Unripe
        } else if t < 0.0 {
            Stage::Turning
        } else if t < self.window {
            Stage::Ripe
        } else {
            Stage::Over
        }
    }

    fn growing(&self) -> bool {
        self.fate == Fate::Growing
    }
}

/// Someone climbing into the high branches, or a close pair bending one down.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Climb {
    who: Id,
    since: f32,
}

pub struct Foray {
    /// How dark the hour has made the hedgerow already, so the outing's own dusk only darkens it
    /// past that.
    hour_dark: f32,
    party: Vec<Id>,
    characters: Vec<Character>,
    items: Vec<Item>,
    phase: Phase,
    light: f32,
    full: f32,
    basket: Vec<&'static str>,
    events: Vec<Event>,
    dice: Dice,
    reduce_motion: bool,
    clock: f32,
    /// Until when the party is busy picking.
    busy_until: f32,
    climb: Option<Climb>,
    /// Something just picked, held up to look at, and since when.
    shown: Option<(usize, f32)>,
    /// Where the leader was last tick, for the light its walking costs.
    walked_from: Option<(f32, f32)>,
    dusk_told: bool,
    /// Who opens what, and what the party is like, read once from who came.
    little: Option<Id>,
    climber: Option<Id>,
    close_pair: bool,
    inspector: Option<Id>,
    spotters: Vec<(Id, f32)>,
    sweetheart: Option<Id>,
    /// How much light walking costs this party, per pixel.
    walk_cost: f32,
    /// How many things the birds have been left, and whether they have said thank you.
    for_birds: u32,
    thanked: bool,
    found_before: Vec<&'static str>,
}

impl Foray {
    /// Sets out with the party, already walking down the lane in `ground`. `found_before` says
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
            seed,
        } = outset;
        let characters: Vec<Character> = party
            .iter()
            .filter_map(|id| ground.character(*id).cloned())
            .collect();
        let mut dice = Dice::new(seed);
        let with = |test: &dyn Fn(&Character) -> bool| {
            party
                .iter()
                .zip(&characters)
                .find(|(_, c)| test(c))
                .map(|(id, _)| *id)
        };
        let little = with(&|c| c.parent.is_some());
        let climber = party
            .iter()
            .zip(&characters)
            .filter(|(_, c)| bold(c))
            .max_by(|a, b| a.1.axes.boldness.total_cmp(&b.1.axes.boldness))
            .map(|(id, _)| *id);
        let close_pair = close_pair && party.len() >= 2;
        let inspector = with(&|c| c.habits.contains(&Habit::LooksFoodOver));
        let spotters: Vec<(Id, f32)> = party
            .iter()
            .zip(&characters)
            .filter(|(_, c)| curious(c))
            .map(|(id, c)| (*id, c.axes.curiosity))
            .collect();
        let sweetheart = with(&sweet);
        let refs: Vec<&Character> = characters.iter().collect();
        let full = LIGHT + influence.light;
        let earlier = (influence.earlier / full).min(0.15);
        let items = stock(
            &refs,
            &found_before,
            drought,
            Openers {
                tucked: little.is_some(),
                high: climber.is_some() || close_pair,
            },
            earlier,
            &mut dice,
        );
        let mut events = Vec::new();
        if let Some(who) = little {
            events.push(Event::Opened {
                who,
                reach: Reach::Tucked,
                pair: false,
            });
        }
        match (climber, close_pair, party.first()) {
            (Some(who), _, _) => events.push(Event::Opened {
                who,
                reach: Reach::High,
                pair: false,
            }),
            (None, true, Some(&who)) => events.push(Event::Opened {
                who,
                reach: Reach::High,
                pair: true,
            }),
            _ => {}
        }
        // The slowest sets the pace.
        let liveliness = characters.iter().map(|c| c.axes.energy).fold(1.0, f32::min);
        Self {
            hour_dark: 0.0,
            party,
            items,
            phase: Phase::Arriving { since: now },
            light: full,
            full,
            basket: Vec::new(),
            events,
            dice,
            reduce_motion: ground.reduce_motion(),
            clock: now,
            busy_until: now,
            climb: None,
            shown: None,
            walked_from: None,
            dusk_told: false,
            little,
            climber,
            close_pair,
            inspector,
            spotters,
            sweetheart,
            walk_cost: WALK_COST / (0.8 + 0.4 * liveliness),
            for_birds: 0,
            thanked: false,
            found_before: FORAGED
                .iter()
                .map(|find| find.id)
                .filter(|id| found_before(id))
                .collect(),
            characters,
        }
    }

    /// Starts this foray partway through a longer day than a foray's own, as a leg of an
    /// expedition: with `light` left of that day's `full`, so everything is as ripe as the
    /// expedition's hour makes it (what has gone over by then is gone already), and with the
    /// basket carried in already in it.
    pub fn partway(mut self, light: f32, full: f32, basket: Vec<&'static str>) -> Self {
        self.light = light;
        self.full = full;
        self.basket = basket;
        let gone = self.gone();
        for item in &mut self.items {
            if gone >= item.ripe_at + item.window + LINGER {
                item.fate = Fate::Gone;
            }
        }
        self
    }

    /// The light left, on the day's own scale.
    pub fn light(&self) -> f32 {
        self.light
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn party(&self) -> &[Id] {
        &self.party
    }

    pub fn basket(&self) -> &[&'static str] {
        &self.basket
    }

    /// How much of the light is left, from 0 to 1.
    pub fn light_left(&self) -> f32 {
        (self.light / self.full).clamp(0.0, 1.0)
    }

    /// How much of the outing has gone, from 0 to 1: the clock everything ripens by.
    fn gone(&self) -> f32 {
        (1.0 - self.light / self.full).clamp(0.0, 1.0)
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// How dark the hour has made the hedgerow already: the outing's dusk adds only what is more.
    pub fn set_hour_dark(&mut self, darkness: f32) {
        self.hour_dark = darkness;
    }

    /// The patch the party is at, if it is at one.
    pub fn at(&self) -> Option<usize> {
        match self.phase {
            Phase::At { patch } => Some(patch),
            _ => None,
        }
    }

    /// What a canny forager would do next, for the renders and the tests: pick the best thing
    /// ripe where it is, making room for it if it is worth more than the least thing in the
    /// basket; otherwise go wherever something worth having turns ripe soonest, the rarer first
    /// between near equals.
    pub fn canny(&self, ground: &Playground) -> Move {
        let gone = self.gone();
        let worth = |find: &Find| {
            let new = !self.found_before.contains(&find.id) && !self.basket.contains(&find.id);
            find.tier as u8 * 2 + u8::from(new)
        };
        let least = self
            .basket
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| finds::find(id).map(|find| (slot, worth(find))))
            .min_by_key(|(_, value)| *value);
        let full = self.basket.len() >= BASKET;
        let wanted =
            |item: &Item| !full || least.is_some_and(|(_, value)| worth(item.find) > value);
        let reachable = |item: &&Item| {
            item.revealed && item.growing() && self.picker(item.slot.reach).is_some()
        };
        if let Phase::At { patch } = self.phase {
            let best = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.patch == patch && reachable(item))
                .filter(|(_, item)| item.stage(gone) == Stage::Ripe && wanted(item))
                .max_by_key(|(_, item)| worth(item.find));
            if let Some((index, _)) = best {
                return match (full, least) {
                    (true, Some((slot, _))) => Move::PutBack(slot),
                    _ => Move::Pick(index),
                };
            }
        }
        let from = self
            .party
            .first()
            .and_then(|id| ground.position(*id))
            .unwrap_or((0.0, 172.0));
        let next = self
            .items
            .iter()
            .filter(reachable)
            .filter(|item| item.stage(gone) <= Stage::Ripe && wanted(item))
            .min_by(|a, b| {
                let key = |item: &Item| {
                    (item.ripe_at - gone).max(0.0) * 10.0 - f32::from(worth(item.find)) * 0.08
                        + distance(from, PATCHES[item.patch].stand) * 0.001
                };
                key(a).total_cmp(&key(b))
            })
            .map(|item| item.patch);
        match next {
            Some(patch) if self.at() != Some(patch) => Move::Go(patch),
            _ => Move::Wait,
        }
    }

    /// What grows at an item, and where, for naming it.
    pub fn item(&self, item: usize) -> Option<(Plant, usize, (f32, f32))> {
        self.items
            .get(item)
            .map(|item| (item.slot.plant, item.patch, item.slot.at))
    }

    /// How ripe an item looks now, if it can be seen growing.
    pub fn stage_of(&self, item: usize) -> Option<Stage> {
        let item = self.items.get(item)?;
        (item.revealed && item.growing()).then(|| item.stage(self.gone()))
    }

    /// What one that looks food over makes of an item: how ripe it really is, if one came.
    pub fn inspected(&self, item: usize) -> Option<Stage> {
        self.inspector?;
        self.stage_of(item)
    }

    /// The thing growing under a point, nearest first: only what can be seen, and what this
    /// party can get at.
    pub fn item_at(&self, x: f32, y: f32) -> Option<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.revealed && item.growing())
            .map(|(index, item)| {
                let middle = middle(item);
                (index, distance((x, y), middle))
            })
            .filter(|(_, off)| *off <= 7.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    /// The patch a point is in or near: one of its things, or where whoever picks there stands.
    pub fn patch_at(&self, x: f32, y: f32) -> Option<usize> {
        if let Some(item) = self.item_at(x, y) {
            return Some(self.items[item].patch);
        }
        PATCHES
            .iter()
            .enumerate()
            .flat_map(|(index, patch)| {
                patch
                    .slots
                    .iter()
                    .map(move |slot| (index, distance((x, y), slot.at)))
                    .chain(std::iter::once((
                        index,
                        distance((x, y), (patch.stand.0, patch.stand.1 - 8.0)),
                    )))
            })
            .filter(|(_, off)| *off <= 16.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    /// The person chose a patch: the party goes over to it. A sweetheart leaving a patch with
    /// ripe things still on it leaves them for the birds.
    pub fn choose(&mut self, ground: &mut Playground, patch: usize, now: f32) {
        let ready = match self.phase {
            Phase::Choosing => true,
            Phase::At { patch: here } | Phase::Going { patch: here } => here != patch,
            _ => false,
        };
        let Some(place) = PATCHES.get(patch) else {
            return;
        };
        if !ready {
            return;
        }
        self.climb_down(ground);
        let mut sweetheart_steps = Vec::new();
        if let (Phase::At { patch: here }, Some(who)) = (self.phase, self.sweetheart) {
            let gone = self.gone();
            let mut left = 0;
            let mut at = now + 1.4;
            for item in &mut self.items {
                if item.patch == here
                    && item.revealed
                    && item.growing()
                    && item.stage(gone) == Stage::Ripe
                {
                    item.fate = Fate::ForBirds { at };
                    at += 1.1;
                    left += 1;
                }
            }
            if left > 0 {
                self.for_birds += left;
                self.events.push(Event::ForTheBirds { who, count: left });
                let mut fond = Beat::new(Gesture::Beg, ExpressionKind::Affectionate, 1.0);
                fond.cue = Some(Cue::Heart);
                sweetheart_steps.push(Step::FaceX(PATCHES[here].stand.0));
                sweetheart_steps.push(Step::Beat(fond));
            }
        }
        ground.reserve(self.party.clone());
        let face =
            place.slots.iter().map(|slot| slot.at.0).sum::<f32>() / place.slots.len().max(1) as f32;
        for (index, id) in self.party.clone().into_iter().enumerate() {
            let to = if index == 0 {
                place.stand
            } else {
                beside(ground, place.stand, face)
            };
            let mut steps = if Some(id) == self.sweetheart {
                std::mem::take(&mut sweetheart_steps)
            } else {
                Vec::new()
            };
            steps.push(Step::Walk { to });
            steps.push(Step::FaceX(face + 0.5));
            ground.direct(id, steps, now);
        }
        self.walked_from = self.party.first().and_then(|id| ground.position(*id));
        self.phase = Phase::Going { patch };
    }

    /// Picks an item at the patch the party is at, as it is right now.
    pub fn pick(&mut self, ground: &mut Playground, index: usize, now: f32) {
        let Phase::At { patch } = self.phase else {
            return;
        };
        if now < self.busy_until {
            return;
        }
        let gone = self.gone();
        let Some(item) = self.items.get(index) else {
            return;
        };
        if item.patch != patch || !item.revealed || !item.growing() {
            return;
        }
        let stage = item.stage(gone);
        let (reach, at, find, window, ripe_at) = (
            item.slot.reach,
            item.slot.at,
            item.find,
            item.window,
            item.ripe_at,
        );
        let Some(picker) = self.picker(reach) else {
            return;
        };
        if self.basket.len() >= BASKET && stage >= Stage::Ripe {
            self.events.push(Event::BasketFull);
            return;
        }
        // One that looks food over has a look first: nothing is picked before its time.
        if let Some(inspector) = self.inspector
            && stage < Stage::Ripe
        {
            self.events.push(Event::LookedOver {
                who: inspector,
                item: index,
            });
            if let Some(character) = ground.character(inspector) {
                let look = character
                    .flourish(formiga_core::HabitCue::Meal)
                    .unwrap_or_else(|| Beat::new(ActionKind::Eat, ExpressionKind::Curious, 1.0));
                ground.direct(inspector, vec![Step::FaceX(at.0), Step::Beat(look)], now);
            }
            self.busy_until = now + PICK_SECS;
            return;
        }
        self.light -= PICK_COST;
        let climbing = reach == Reach::High;
        if climbing {
            self.light -= CLIMB_COST;
        }
        self.reach_for(ground, picker, reach, at, now);
        let gentle = self.gentleness(picker);
        let kept = stage == Stage::Ripe
            || (stage == Stage::Over && gone - ripe_at < window + gentle * window);
        match stage {
            Stage::Unripe | Stage::Turning => {
                self.events.push(Event::NotYet { item: index });
            }
            _ if kept => {
                let plenty = find.tier == Tier::Common && self.basket.contains(&find.id);
                if let Some(inspector) = self.inspector
                    && plenty
                    && self.dice.chance(NIBBLE)
                {
                    self.items[index].fate = Fate::Taken;
                    self.events.push(Event::Ate {
                        who: inspector,
                        item: index,
                    });
                    let munch = Beat::new(ActionKind::Eat, ExpressionKind::Joy, 1.2);
                    ground.direct(inspector, vec![Step::Beat(munch)], now);
                } else {
                    self.items[index].fate = Fate::Taken;
                    self.basket.push(find.id);
                    if stage == Stage::Over {
                        self.events.push(Event::Saved {
                            who: picker,
                            item: index,
                        });
                    }
                    self.events.push(Event::Picked {
                        find: find.id,
                        item: index,
                    });
                    self.shown = Some((index, now));
                    let delight = self
                        .character(picker)
                        .map(|character| delighted(character, find.tier))
                        .unwrap_or_default();
                    if !climbing {
                        let mut steps: Vec<Step> =
                            vec![Step::Beat(pick_beat(reach, at, ground, picker))];
                        steps.extend(delight.into_iter().map(Step::Beat));
                        ground.direct(picker, steps, now);
                    }
                }
            }
            // Past its best: it squashes, or one that looks food over eats it.
            _ => match self.inspector {
                Some(inspector) => {
                    self.items[index].fate = Fate::Taken;
                    self.events.push(Event::Ate {
                        who: inspector,
                        item: index,
                    });
                }
                None => {
                    self.items[index].fate = Fate::Fell { since: now };
                    self.events.push(Event::Spoilt { item: index });
                }
            },
        }
        self.busy_until = now + if climbing { CLIMB_SECS } else { PICK_SECS };
        if self.light <= 0.0 {
            self.leave(ground, Ending::Dusk, now);
        }
    }

    /// Puts something from the basket back, to make room: it is left on the lane for the birds.
    pub fn put_back(&mut self, index: usize) {
        if matches!(self.phase, Phase::Leaving { .. } | Phase::Over) || index >= self.basket.len() {
            return;
        }
        let find = self.basket.remove(index);
        self.events.push(Event::PutBack { find });
    }

    /// Calls the foray to an end: everyone heads home with what is in the basket.
    pub fn head_home(&mut self, ground: &mut Playground, now: f32) {
        if !matches!(self.phase, Phase::Leaving { .. } | Phase::Over) {
            self.leave(ground, Ending::Chose, now);
        }
    }

    fn leave(&mut self, ground: &mut Playground, why: Ending, now: f32) {
        self.climb_down(ground);
        ground.reserve(self.party.clone());
        for (index, id) in self.party.iter().enumerate() {
            let tired = why == Ending::Dusk
                && ground
                    .character(*id)
                    .is_some_and(|c| c.kind == TemperamentKind::Lazybones);
            let mut steps = Vec::new();
            if tired {
                steps.push(Step::Beat(Beat::new(
                    Gesture::Yawn,
                    ExpressionKind::Yawning,
                    1.0,
                )));
            }
            steps.push(Step::Walk {
                to: (-30.0 - 20.0 * index as f32, 174.0),
            });
            ground.direct(*id, steps, now);
        }
        self.events.push(Event::Leaving(why));
        self.phase = Phase::Leaving { since: now, why };
    }

    pub fn tick(&mut self, ground: &mut Playground, now: f32) {
        let dt = (now - self.clock).clamp(0.0, 0.1);
        self.clock = now;
        let out = matches!(
            self.phase,
            Phase::Choosing | Phase::Going { .. } | Phase::At { .. }
        );
        if out {
            self.light -= LIGHT_PER_SEC * dt;
            // Walking from one place to the next takes its own light, step by step, so things
            // ripen on the way.
            if let Some(leader) = self.party.first()
                && let Some(here) = ground.position(*leader)
            {
                if let Some(from) = self.walked_from
                    && matches!(self.phase, Phase::Going { .. })
                {
                    self.light -= distance(from, here) * self.walk_cost;
                }
                self.walked_from = Some(here);
            }
        }
        let gone = self.gone();
        self.ripen(gone, now);
        if out {
            self.spot(ground, dt, now);
            self.tell_of_ripening(gone);
            self.thank(ground, gone, now);
        }
        self.lift(ground, now);
        if !self.dusk_told && gone >= DUSK && out {
            self.dusk_told = true;
            self.events.push(Event::Dusk);
        }
        let leader = self.party.first().copied();
        let busy = |ground: &Playground| leader.is_some_and(|id| ground.busy(id));
        match self.phase {
            Phase::Arriving { since } => {
                if !busy(ground) || now - since > 6.0 {
                    ground.reserve(self.party.clone());
                    self.phase = Phase::Choosing;
                }
            }
            Phase::Going { patch } => {
                let walking = leader.is_some_and(|id| ground.walking(id));
                if !walking {
                    self.arrive(patch);
                }
            }
            Phase::Leaving { since, .. } => {
                let gone = self.party.iter().all(|id| !ground.busy(*id));
                if gone || now - since > 8.0 {
                    self.phase = Phase::Over;
                }
            }
            _ => {}
        }
        if self.light <= 0.0 && out {
            self.leave(ground, Ending::Dusk, now);
        }
        if self.shown.is_some_and(|(_, since)| now - since > SHOW_SECS) {
            self.shown = None;
        }
    }

    /// At a patch: anything hiding there is found, looking closely.
    fn arrive(&mut self, patch: usize) {
        for (index, item) in self.items.iter_mut().enumerate() {
            if item.patch == patch && !item.revealed && item.growing() {
                item.revealed = true;
                self.events.push(Event::Uncovered { item: index });
            }
        }
        self.phase = Phase::At { patch };
    }

    /// Everything goes over in its time and drops; what was left for the birds, they take.
    fn ripen(&mut self, gone: f32, now: f32) {
        for (index, item) in self.items.iter_mut().enumerate() {
            match item.fate {
                Fate::Growing if gone >= item.ripe_at + item.window + LINGER => {
                    item.fate = Fate::Fell { since: now };
                    if item.revealed {
                        self.events.push(Event::Dropped { item: index });
                    }
                }
                Fate::Fell { since } if now - since > FALL_SECS + FALLEN_SECS => {
                    item.fate = Fate::Gone;
                }
                Fate::ForBirds { at } if now >= at => item.fate = Fate::Gone,
                _ => {}
            }
        }
    }

    /// Someone curious, with nothing else to do, may spot something hiding.
    fn spot(&mut self, ground: &mut Playground, dt: f32, now: f32) {
        if self.spotters.is_empty() || now < self.busy_until {
            return;
        }
        for index in 0..self.items.len() {
            let item = &self.items[index];
            if item.revealed || !item.growing() {
                continue;
            }
            let at = item.slot.at;
            for (who, curiosity) in self.spotters.clone() {
                let near = ground
                    .position(who)
                    .is_some_and(|here| distance(here, at) < 230.0);
                let chance = 1.0 - (-SPOTTING * curiosity * dt).exp();
                if near && !ground.walking(who) && self.dice.chance(chance) {
                    self.items[index].revealed = true;
                    self.events.push(Event::Spotted { who, item: index });
                    let mut point = Beat::new(Gesture::Reach, ExpressionKind::Curious, 0.9);
                    point.cue = Some(Cue::Exclaim);
                    ground.direct(who, vec![Step::FaceX(at.0), Step::Beat(point)], now);
                    break;
                }
            }
        }
    }

    /// Someone curious says when something rare is nearly ripe, and where: the hedgerow's signs.
    fn tell_of_ripening(&mut self, gone: f32) {
        let Some(&(who, _)) = self.spotters.first() else {
            return;
        };
        for (index, item) in self.items.iter_mut().enumerate() {
            if !item.told
                && item.growing()
                && item.find.tier >= Tier::Rare
                && item.stage(gone) == Stage::Turning
            {
                item.told = true;
                item.revealed = true;
                self.events.push(Event::Ripening { who, item: index });
            }
        }
    }

    /// Once a sweetheart has left the birds enough, a robin brings something back and leaves it
    /// at the party's feet: an extra chance at something, never the only one.
    fn thank(&mut self, ground: &mut Playground, gone: f32, now: f32) {
        let (Some(who), Phase::At { patch }) = (self.sweetheart, self.phase) else {
            return;
        };
        if self.thanked || self.for_birds < THANKS || now < self.busy_until {
            return;
        }
        self.thanked = true;
        let party: Vec<&Character> = self.characters.iter().collect();
        let found = &self.found_before;
        let pool: Vec<&'static Find> = FORAGED
            .iter()
            .filter(|find| find.tier < Tier::Exceptional)
            .collect();
        let weight = |find: &Find| {
            let novelty = if found.contains(&find.id) {
                1.0
            } else {
                NOVELTY
            };
            find.tier.weight() * drawn_to(find.leanings, &party) * novelty
        };
        let weights: Vec<f32> = pool.iter().map(|find| weight(find)).collect();
        let Some(chosen) = self.dice.weighted(&weights).map(|index| pool[index]) else {
            return;
        };
        let Some(plant) = Plant::of(chosen.id) else {
            return;
        };
        let stand = PATCHES[patch].stand;
        let side = if stand.0 > 190.0 { -1.0 } else { 1.0 };
        let at = (stand.0 + side * 12.0, stand.1 + 3.0);
        self.items.push(Item {
            slot: Slot {
                plant,
                at,
                reach: Reach::Within,
                hidden: false,
            },
            patch,
            find: chosen,
            ripe_at: gone,
            window: 0.25,
            revealed: true,
            told: true,
            fate: Fate::Growing,
            gift: true,
        });
        self.events.push(Event::Thanked {
            who,
            item: self.items.len() - 1,
        });
        let mut fond = Beat::new(Gesture::Cheer, ExpressionKind::Affectionate, 1.2);
        fond.cue = Some(Cue::Heart);
        ground.direct(who, vec![Step::FaceX(at.0), Step::Beat(fond)], now);
    }

    /// Who picks something reached this way: a little one for the tucked-in places, a climber
    /// (or the first of a close pair) for the high ones, and otherwise the one who leads.
    fn picker(&self, reach: Reach) -> Option<Id> {
        match reach {
            Reach::Within => self.party.first().copied(),
            Reach::Tucked => self.little,
            Reach::High => self.climber.or_else(|| {
                self.close_pair
                    .then(|| self.party.first().copied())
                    .flatten()
            }),
        }
    }

    fn character(&self, id: Id) -> Option<&Character> {
        self.party
            .iter()
            .position(|member| *member == id)
            .and_then(|index| self.characters.get(index))
    }

    /// How much longer than its window ripeness lasts in this one's paws, as a share of it.
    fn gentleness(&self, picker: Id) -> f32 {
        self.character(picker).map_or(0.0, |c| {
            GENTLE * (1.0 - c.axes.impulsiveness).clamp(0.0, 1.0)
        })
    }

    /// The picker reaching for something: up into the hedge, down into the tangle, or up into
    /// the high branches, climbing, or with a close friend bending the branch down.
    fn reach_for(
        &mut self,
        ground: &mut Playground,
        picker: Id,
        reach: Reach,
        at: (f32, f32),
        now: f32,
    ) {
        if reach != Reach::High {
            return;
        }
        if self.climber == Some(picker) {
            self.climb = Some(Climb {
                who: picker,
                since: now,
            });
            let up = Beat::new(
                Gesture::Balance,
                ExpressionKind::Determined,
                CLIMB_SECS * 0.5,
            );
            let reaching = Beat::new(Gesture::Reach, ExpressionKind::Joy, CLIMB_SECS * 0.5);
            ground.direct(
                picker,
                vec![Step::FaceX(at.0), Step::Beat(up), Step::Beat(reaching)],
                now,
            );
        } else {
            // Bent down between the two of them: the other comes to its side to help.
            let heave = Beat::new(Gesture::Heave, ExpressionKind::Determined, CLIMB_SECS);
            for id in self.party.clone() {
                let mut steps = Vec::new();
                if id != picker
                    && let Some(here) = ground.position(picker)
                {
                    steps.push(Step::Walk {
                        to: beside(ground, here, at.0),
                    });
                }
                steps.extend([Step::FaceX(at.0), Step::Beat(heave)]);
                ground.direct(id, steps, now);
            }
        }
    }

    /// Raises a climber into the branches and brings it down again: smoothly, or with reduced
    /// motion, up and then down in two cuts.
    fn lift(&mut self, ground: &mut Playground, now: f32) {
        let Some(climb) = self.climb else {
            return;
        };
        let t = (now - climb.since) / CLIMB_SECS;
        if t >= 1.0 {
            ground.set_lift(climb.who, 0.0);
            self.climb = None;
            return;
        }
        ground.set_lift(climb.who, climb_height(t, self.reduce_motion));
    }

    fn climb_down(&mut self, ground: &mut Playground) {
        if let Some(climb) = self.climb.take() {
            ground.set_lift(climb.who, 0.0);
        }
    }

    /// Everything growing that can be seen, each standing on its patch's row among everyone, and
    /// a robin by whatever was left for it.
    pub fn produce(&self, now: f32) -> Vec<Prop> {
        let gone = self.gone();
        let mut props = Vec::new();
        for item in &self.items {
            if !item.revealed {
                continue;
            }
            let patch = &PATCHES[item.patch];
            let lying = |slot: &Slot| {
                let (_, ay) = produce::anchor(slot.plant);
                Slot {
                    at: (slot.at.0, slot.at.1 + (10 - ay) as f32 - 9.0),
                    ..*slot
                }
            };
            match item.fate {
                Fate::Growing => {
                    let slot = if item.gift {
                        lying(&item.slot)
                    } else {
                        item.slot
                    };
                    let base = if item.gift {
                        item.slot.at.1
                    } else {
                        patch.base
                    };
                    props.push(produce::prop(
                        &slot,
                        base,
                        item.stage(gone),
                        now,
                        self.reduce_motion,
                    ));
                }
                Fate::Fell { since } => {
                    let t = if self.reduce_motion {
                        1.0
                    } else {
                        ((now - since) / FALL_SECS).clamp(0.0, 1.0)
                    };
                    let landed = lying(&Slot {
                        at: (item.slot.at.0, patch.floor),
                        ..item.slot
                    });
                    let y = item.slot.at.1 + (landed.at.1 - item.slot.at.1) * t * t;
                    let slot = Slot {
                        at: (item.slot.at.0, y),
                        ..item.slot
                    };
                    props.push(produce::prop(&slot, patch.floor, Stage::Over, now, true));
                }
                Fate::ForBirds { at } => {
                    let slot = if item.gift {
                        lying(&item.slot)
                    } else {
                        item.slot
                    };
                    props.push(produce::prop(&slot, patch.base, Stage::Ripe, now, true));
                    if at - now < ROBIN_SECS {
                        props.push(robin(
                            item.slot.at,
                            patch.base + 0.5,
                            now,
                            self.reduce_motion,
                        ));
                    }
                }
                Fate::Taken | Fate::Gone => {}
            }
        }
        props
    }

    /// The hedgerow dimming as the light goes, the thing under the pointer marked, something just
    /// picked held up, and the basket.
    pub fn draw(&self, scene: &mut Canvas, hovered: Option<usize>, now: f32) {
        dim(scene, self.light, self.hour_dark);
        if let Some(index) = hovered
            && let Some(item) = self.items.get(index)
            && item.revealed
            && item.growing()
        {
            let here = self.at() == Some(item.patch);
            let color = if here {
                rgb(0xf5d25e)
            } else {
                rgba(0xfdfbf5, 220)
            };
            brackets(scene, middle(item), 5, color);
        }
        if let Some((index, since)) = self.shown
            && let Some(item) = self.items.get(index)
            && let Some(picker) = self.picker(item.slot.reach)
        {
            let _ = picker;
            let rise = if self.reduce_motion {
                8.0
            } else {
                ((now - since) / 0.4).min(1.0) * 8.0
            };
            let (x, y) = item.slot.at;
            let icon = finds::art::icon(item.find.id);
            let (left, top) = (x as i32 - 4, (y - 8.0 - rise) as i32);
            rect(scene, left - 2, top - 2, 13, 13, rgba(0xf6eed8, 230));
            blit(scene, &icon, left, top);
            if item.find.tier >= Tier::Rare {
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

    /// The basket in the top right corner: a slot for each thing it holds, and the light left.
    fn draw_basket(&self, scene: &mut Canvas) {
        let (left, top, slot) = self.basket_frame(scene.width());
        let width = slot * BASKET as i32 + 3;
        rect(scene, left, top, width, slot + 7, rgba(0x2a2018, 150));
        for index in 0..BASKET as i32 {
            let (x, y) = (left + 2 + index * slot, top + 2);
            rect(scene, x, y, slot - 1, slot - 1, rgba(0xf6eed8, 60));
            if let Some(id) = self.basket.get(index as usize) {
                blit(scene, &finds::art::icon(id), x, y);
            }
        }
        let share = self.light_left();
        let bar = ((width - 4) as f32 * share) as i32;
        let gold = mix(rgb(0x6a5a9a), rgb(0xf5d25e), share);
        scene.fill_rect(left + 2, top + slot + 2, bar, 2, gold);
    }
}

/// Which places beyond everyone's reach this party opens.
#[derive(Clone, Copy, Debug)]
struct Openers {
    tucked: bool,
    high: bool,
}

/// What grows where on one foray, and when each ripens. Open places bear by weight, rarer things
/// and things the party is less drawn to less often, undiscovered things more often, and the
/// rarer things only once a foray; tucked and high places, where only some company reaches, always
/// bear. After a dry spell something new is certain, somewhere anyone can reach. Each plant's
/// things ripen spread through its own part of the outing, so a patch is worth coming back to.
fn stock(
    party: &[&Character],
    found_before: &impl Fn(&str) -> bool,
    drought: u32,
    open: Openers,
    earlier: f32,
    dice: &mut Dice,
) -> Vec<Item> {
    let weight = |find: &Find| {
        let novelty = if found_before(find.id) { 1.0 } else { NOVELTY };
        find.tier.weight() * drawn_to(find.leanings, party) * novelty
    };
    let mut items: Vec<Item> = Vec::new();
    for (patch, slot) in super::slots() {
        let find = slot.plant.find();
        let bears = match slot.reach {
            Reach::Tucked => open.tucked,
            Reach::High => open.high,
            Reach::Within => {
                let once =
                    find.tier < Tier::Rare || !items.iter().any(|item| item.find.id == find.id);
                once && dice.chance(1.0 - (-weight(find) / BEARING).exp())
            }
        };
        if !bears {
            continue;
        }
        items.push(Item {
            slot: *slot,
            patch,
            find,
            ripe_at: 0.0,
            window: 0.0,
            revealed: !slot.hidden,
            told: false,
            fate: Fate::Growing,
            gift: false,
        });
    }
    // A long run of nothing new: something new is certainly growing somewhere, the least rare
    // first, in a place anyone can reach.
    let undiscovered: Vec<&'static Find> = FORAGED
        .iter()
        .filter(|find| !found_before(find.id))
        .filter(|find| !items.iter().any(|item| item.find.id == find.id))
        .collect();
    if drought >= DROUGHT
        && let Some(least) = undiscovered.iter().map(|find| find.tier).min()
    {
        let candidates: Vec<&'static Find> = undiscovered
            .into_iter()
            .filter(|find| find.tier == least)
            .collect();
        let weights: Vec<f32> = candidates.iter().map(|find| weight(find)).collect();
        if let Some(new) = dice.weighted(&weights).map(|index| candidates[index])
            && let Some(plant) = Plant::of(new.id)
        {
            let places: Vec<(usize, &'static Slot)> = super::slots()
                .filter(|(_, slot)| slot.plant == plant && slot.reach == Reach::Within)
                .collect();
            let (patch, slot) = places[(dice.unit() * places.len() as f32) as usize % places.len()];
            items.retain(|item| !(item.slot.at == slot.at && item.patch == patch));
            items.push(Item {
                slot: *slot,
                patch,
                find: new,
                ripe_at: 0.0,
                window: 0.0,
                revealed: !slot.hidden,
                told: false,
                fate: Fate::Growing,
                gift: false,
            });
        }
    }
    // When each ripens: each plant's things spread out through its part of the outing.
    for plant in Plant::ALL {
        let ((early, late), window) = plant.rhythm();
        let shift = match plant.find().tier {
            Tier::Rare => earlier,
            Tier::Exceptional => earlier * 0.5,
            _ => 0.0,
        };
        let mine: Vec<usize> = (0..items.len())
            .filter(|index| items[*index].slot.plant == plant)
            .collect();
        let count = mine.len();
        let mut order: Vec<usize> = (0..count).collect();
        for index in (1..count).rev() {
            let other = (dice.unit() * (index + 1) as f32) as usize % (index + 1);
            order.swap(index, other);
        }
        for (place, index) in order.into_iter().zip(mine) {
            let span = (late - early) / count as f32;
            let at = early + span * (place as f32 + dice.unit());
            items[index].ripe_at = (at - shift).max(0.0);
            items[index].window = window * dice.range(0.85, 1.15);
        }
    }
    items
}

/// Whether a companion will climb for the high branches.
fn bold(character: &Character) -> bool {
    character.axes.boldness >= 0.65
}

/// Whether a companion notices what is hiding, and says when something rare is nearly ripe.
fn curious(character: &Character) -> bool {
    character.kind == TemperamentKind::Explorer || character.axes.curiosity >= 0.6
}

/// Whether a companion would leave the ripe things for the birds.
fn sweet(character: &Character) -> bool {
    character.kind == TemperamentKind::Sweetheart || character.axes.affection >= 0.85
}

/// How high a climber is, `t` of the way through a climb: up, held while it picks, and down.
/// With reduced motion, two cuts: up, then down.
fn climb_height(t: f32, reduce_motion: bool) -> f32 {
    if reduce_motion {
        return if (0.15..0.85).contains(&t) { LIFT } else { 0.0 };
    }
    let shape = if t < 0.3 {
        t / 0.3
    } else if t > 0.7 {
        (1.0 - t) / 0.3
    } else {
        1.0
    };
    LIFT * shape.clamp(0.0, 1.0)
}

/// How the picker reaches for something, by where it is: down into a tangle, or up.
fn pick_beat(reach: Reach, at: (f32, f32), ground: &Playground, picker: Id) -> Beat {
    let feet = ground.position(picker).map_or(150.0, |(_, y)| y);
    if reach == Reach::Tucked || at.1 > feet - 18.0 {
        Beat::new(Gesture::Crouch, ExpressionKind::Focused, PICK_SECS)
    } else {
        Beat::new(Gesture::Reach, ExpressionKind::Focused, PICK_SECS)
    }
}

/// Where a second companion stands while the first picks: beside it, on the side away from what
/// it is picking, unless that is off the lane.
fn beside(ground: &Playground, stand: (f32, f32), face: f32) -> (f32, f32) {
    let side = if face > stand.0 { -BESIDE } else { BESIDE };
    ground.beside_point((stand.0, stand.1 + 4.0), side)
}

/// The middle of something as drawn, for pointing at it.
fn middle(item: &Item) -> (f32, f32) {
    let (x, y) = item.slot.at;
    let (_, ay) = produce::anchor(item.slot.plant);
    if item.gift {
        (x, y - 4.0)
    } else if ay < 5 {
        // Hanging: its middle is below where it hangs from.
        (x, y + 5.0)
    } else {
        (x, y - 4.0)
    }
}

/// Four little corners round a point: the thing the pointer is on.
fn brackets(scene: &mut Canvas, (x, y): (f32, f32), half: i32, color: Rgba) {
    let (x, y) = (x as i32, y as i32);
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let (cx, cy) = (x + sx * half, y + sy * half);
        put(scene, cx, cy, color);
        put(scene, cx - sx, cy, color);
        put(scene, cx, cy - sy, color);
    }
}

/// A robin, come for what was left for it, by `at`: hopping, or sat still with reduced motion.
fn robin(at: (f32, f32), base: f32, now: f32, reduce_motion: bool) -> Prop {
    const ROWS: [&str; 5] = [".bbb..", "bBkbb.", "rRrbbt", ".rrbb.", "..y.y."];
    let mut sprite = Canvas::new(6, 5);
    for (y, row) in ROWS.iter().enumerate() {
        for (x, code) in row.bytes().enumerate() {
            let color = match code {
                b'b' => rgb(0x6a4a30),
                b'B' => rgb(0x8a6a48),
                b'k' => rgb(0x1a1210),
                b'r' => rgb(0xd8602c),
                b'R' => rgb(0xf08a4a),
                b't' => rgb(0x4a3020),
                b'y' => rgb(0x5a4a3a),
                _ => continue,
            };
            sprite.set(x as i32, y as i32, color);
        }
    }
    let hop = if reduce_motion || (now * 3.0).fract() < 0.7 {
        0
    } else {
        1
    };
    Prop::new(sprite, (at.0 as i32 + 3, at.1 as i32 - 1 - hop), base)
}

/// What sort of forager a companion makes, for the person choosing who to bring: the things it
/// does that others don't, from who it is.
pub fn forager(character: &Character) -> String {
    let mut ways: Vec<&str> = Vec::new();
    if character.parent.is_some() {
        ways.push("small enough for the tucked-away ones");
    }
    if bold(character) {
        ways.push("climbs for the high branches");
    }
    if character.habits.contains(&Habit::LooksFoodOver) {
        ways.push("looks everything over first, and may eat one");
    }
    if curious(character) {
        ways.push("spots what hides under the leaves");
    }
    if sweet(character) {
        ways.push("leaves the rest for the birds, who remember it");
    }
    if character.axes.impulsiveness <= 0.4 {
        ways.push("picks so gently that ripeness lasts in its paws");
    }
    if ways.is_empty() {
        ways.push(if character.axes.energy >= 0.6 {
            "quick between the bushes"
        } else {
            "steady with a basket"
        });
    }
    let said = ways.into_iter().take(2).collect::<Vec<_>>().join(", and ");
    let mut first = said.chars();
    first
        .next()
        .map(|letter| letter.to_uppercase().chain(first).collect::<String>() + ".")
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;
    use crate::finds::CATALOGUE;
    use crate::hedgerow;
    use crate::station::SCENE_WIDTH;
    use std::collections::{BTreeMap, BTreeSet};

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    /// A member of the sample colony by name.
    fn named(cast: &Cast, name: &str) -> Id {
        cast.members
            .iter()
            .find(|member| member.name == name)
            .unwrap_or_else(|| panic!("no {name} in the sample"))
            .id
    }

    fn outing_with(
        cast: &Cast,
        party: Vec<Id>,
        close_pair: bool,
        seed: u64,
    ) -> (Playground, Foray) {
        let mut ground = hedgerow::open(cast, &party, 0.0, &Default::default());
        let outset = Outset {
            party,
            drought: 0,
            close_pair,
            influence: Influence::default(),
            seed,
        };
        let foray = Foray::new(&mut ground, outset, |_| false, 0.0);
        (ground, foray)
    }

    fn run(
        ground: &mut Playground,
        foray: &mut Foray,
        cast: &Cast,
        from: f32,
        to: f32,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let mut now = from;
        while now < to {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            foray.tick(ground, now);
            events.extend(foray.take_events());
        }
        events
    }

    /// Gets the party to a patch and lets it settle there.
    fn go_to(
        ground: &mut Playground,
        foray: &mut Foray,
        cast: &Cast,
        patch: usize,
        now: f32,
    ) -> f32 {
        foray.choose(ground, patch, now);
        let mut later = now;
        while foray.at() != Some(patch) && later < now + 30.0 {
            later += 1.0 / 30.0;
            ground.tick(cast, later);
            foray.tick(ground, later);
        }
        assert_eq!(
            foray.at(),
            Some(patch),
            "never got to {}",
            PATCHES[patch].name
        );
        later + 1.0
    }

    /// Something common growing within anyone's reach at a patch, made to ripen `in` the outing
    /// from now, with a window of a tenth of it.
    fn common_at(foray: &mut Foray, patch: usize, ripe_in: f32) -> usize {
        let gone = foray.gone();
        let index = foray
            .items
            .iter()
            .position(|item| {
                item.patch == patch
                    && item.growing()
                    && item.slot.reach == Reach::Within
                    && item.find.tier == Tier::Common
            })
            .unwrap_or_else(|| {
                // Plant one if the dice left the patch bare.
                let slot = *PATCHES[patch]
                    .slots
                    .iter()
                    .find(|slot| slot.reach == Reach::Within)
                    .unwrap();
                foray.items.push(Item {
                    slot,
                    patch,
                    find: slot.plant.find(),
                    ripe_at: 0.0,
                    window: 0.0,
                    revealed: true,
                    told: false,
                    fate: Fate::Growing,
                    gift: false,
                });
                foray.items.len() - 1
            });
        let item = &mut foray.items[index];
        item.revealed = true;
        item.ripe_at = gone + ripe_in;
        item.window = 0.1;
        index
    }

    #[test]
    fn ripeness_runs_unripe_turning_ripe_over_and_then_it_drops() {
        let cast = sample();
        let (_, mut foray) = outing_with(&cast, vec![named(&cast, "Mochi")], false, 1);
        let index = common_at(&mut foray, 0, 0.4);
        let item = foray.items[index].clone();
        assert_eq!(item.stage(0.2), Stage::Unripe);
        assert_eq!(item.stage(0.4 - TURNING / 2.0), Stage::Turning);
        assert_eq!(item.stage(0.45), Stage::Ripe);
        assert_eq!(item.stage(0.52), Stage::Over);
        foray.light = foray.full * (1.0 - (0.4 + 0.1 + LINGER + 0.01));
        foray.ripen(foray.gone(), 10.0);
        assert!(matches!(foray.items[index].fate, Fate::Fell { .. }));
        foray.ripen(foray.gone(), 10.0 + FALL_SECS + FALLEN_SECS + 0.1);
        assert_eq!(foray.items[index].fate, Fate::Gone);
    }

    #[test]
    fn picking_ripe_keeps_it_too_early_leaves_it_and_over_loses_only_that_one() {
        let cast = sample();
        // Button: no habit of looking food over, and middling patience.
        let (mut ground, mut foray) = outing_with(&cast, vec![named(&cast, "Button")], false, 2);
        run(&mut ground, &mut foray, &cast, 0.0, 6.0);
        let now = go_to(&mut ground, &mut foray, &cast, 0, 6.0);
        let item = common_at(&mut foray, 0, 0.3);
        let light = foray.light;
        foray.pick(&mut ground, item, now);
        assert!(foray.take_events().contains(&Event::NotYet { item }));
        assert!(foray.basket().is_empty());
        assert!(foray.items[item].growing(), "too early just leaves it");
        assert!(
            foray.light < light - PICK_COST * 0.9,
            "a picking spends light"
        );

        let now = now + 1.0;
        foray.items[item].ripe_at = foray.gone() - 0.02;
        foray.pick(&mut ground, item, now);
        assert_eq!(foray.basket(), &["blackberries"]);
        assert!(!foray.items[item].growing());

        let now = now + 1.0;
        let over = common_at(&mut foray, 0, 0.0);
        // Past even the gentlest paws, and not yet dropped.
        foray.items[over].ripe_at = foray.gone() - 0.1 * (1.0 + GENTLE) - 0.02;
        assert_eq!(foray.stage_of(over), Some(Stage::Over));
        foray.pick(&mut ground, over, now);
        assert!(foray.take_events().contains(&Event::Spoilt { item: over }));
        assert!(!foray.items[over].growing(), "picked over, it is lost");
        assert_eq!(
            foray.basket(),
            &["blackberries"],
            "nothing in the basket is lost"
        );
    }

    #[test]
    fn nothing_can_be_picked_from_anywhere_but_where_the_party_is() {
        let cast = sample();
        let (mut ground, mut foray) = outing_with(&cast, vec![named(&cast, "Button")], false, 3);
        run(&mut ground, &mut foray, &cast, 0.0, 6.0);
        let item = common_at(&mut foray, 6, -0.02);
        foray.pick(&mut ground, item, 6.0);
        assert!(foray.basket().is_empty(), "picked from the lane");
        let now = go_to(&mut ground, &mut foray, &cast, 0, 6.0);
        foray.items[item].ripe_at = foray.gone() - 0.02;
        foray.pick(&mut ground, item, now);
        assert!(
            foray.basket().is_empty(),
            "picked the crab apple from the bramble"
        );
        let now = go_to(&mut ground, &mut foray, &cast, 6, now);
        foray.items[item].ripe_at = foray.gone() - 0.02;
        foray.pick(&mut ground, item, now);
        assert_eq!(foray.basket().len(), 1);
    }

    /// What grows on a foray with only `character` along, as `Foray::new` stocks it.
    fn grown(
        character: &Character,
        found: impl Fn(&str) -> bool,
        drought: u32,
        dice: &mut Dice,
    ) -> Vec<Item> {
        let open = Openers {
            tucked: character.parent.is_some(),
            high: bold(character),
        };
        stock(&[character], &found, drought, open, 0.0, dice)
    }

    #[test]
    fn the_rarer_things_are_ripe_briefly_and_the_clover_only_in_the_last_of_the_light() {
        let cast = sample();
        let mut clovers = 0;
        let mut dice = Dice::new(14);
        for member in &cast.members {
            let character = Character::of(member);
            for _ in 0..60 {
                for item in grown(&character, |_| false, 0, &mut dice) {
                    let ((_, _), window) = item.slot.plant.rhythm();
                    assert!(
                        item.window <= window * 1.16,
                        "{:?} ripe too long",
                        item.slot.plant
                    );
                    match item.find.tier {
                        Tier::Rare => assert!(item.window < 0.08 && item.ripe_at >= 0.4),
                        Tier::Exceptional => {
                            clovers += 1;
                            assert!(
                                item.ripe_at >= DUSK,
                                "the clover showed at {}",
                                item.ripe_at
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        assert!(clovers > 0, "the clover never grew");
    }

    #[test]
    fn nothing_rummaged_grows_at_the_hedgerow() {
        let cast = sample();
        let mut dice = Dice::new(15);
        for member in &cast.members {
            let character = Character::of(member);
            for drought in 0..4 {
                for item in grown(&character, |_| false, drought, &mut dice) {
                    assert!(
                        finds::is_foraged(item.find.id),
                        "{} grew here",
                        item.find.id
                    );
                    assert!(CATALOGUE.iter().all(|find| find.id != item.find.id));
                }
            }
        }
    }

    /// The completeness guarantee: with only ever one companion, keep going and everything is
    /// picked in the end, whoever that companion is, picking only what its own paws can reach.
    #[test]
    fn anyone_on_their_own_picks_everything_in_the_end() {
        let cast = sample();
        for member in &cast.members {
            let character = Character::of(member);
            let mut dice = Dice::new(member.id);
            let mut found: BTreeSet<&str> = BTreeSet::new();
            let mut drought = 0;
            let mut forays = 0;
            while found.len() < FORAGED.len() {
                forays += 1;
                assert!(
                    forays <= 60,
                    "{} after {forays} forays had picked only {found:?}",
                    member.name
                );
                // A middling foray: a basketful, the new things first.
                let mut picked: Vec<&'static str> =
                    grown(&character, |id| found.contains(id), drought, &mut dice)
                        .into_iter()
                        .map(|item| item.find.id)
                        .collect();
                picked.sort_by_key(|id| found.contains(id));
                let mut new = false;
                for id in picked.into_iter().take(BASKET) {
                    new |= found.insert(id);
                }
                drought = if new { 0 } else { drought + 1 };
            }
        }
    }

    #[test]
    fn undiscovered_things_grow_more_often_and_after_a_dry_spell_something_new_is_certain() {
        let cast = sample();
        let character = Character::of(&cast.members[0]);
        let count = |found: &dyn Fn(&str) -> bool, drought: u32, id: &str| {
            let mut dice = Dice::new(17);
            (0..300)
                .filter(|_| {
                    grown(&character, found, drought, &mut dice)
                        .iter()
                        .any(|item| item.find.id == id)
                })
                .count()
        };
        let all = |_: &str| true;
        let none = |_: &str| false;
        assert!(count(&none, 0, "golden_chanterelle") > count(&all, 0, "golden_chanterelle"));
        // Everything found but the clover: after two dry forays, it is certainly growing.
        let but_clover = |id: &str| id != "four_leaf_clover";
        let wet = count(&but_clover, 0, "four_leaf_clover");
        assert!(wet < 300, "the clover always grows anyway");
        assert_eq!(count(&but_clover, DROUGHT, "four_leaf_clover"), 300);
    }

    #[test]
    fn who_comes_along_changes_what_grows() {
        let cast = sample();
        let tally = |name: &str| {
            let member = cast
                .members
                .iter()
                .find(|member| member.name == name)
                .unwrap();
            let character = Character::of(member);
            let mut dice = Dice::new(16);
            let mut seen: BTreeMap<&str, u32> = BTreeMap::new();
            for _ in 0..400 {
                for item in grown(&character, |_| false, 0, &mut dice) {
                    if item.slot.reach == Reach::Within {
                        *seen.entry(item.find.id).or_default() += 1;
                    }
                }
            }
            seen
        };
        let (mochi, biscuit) = (tally("Mochi"), tally("Biscuit"));
        assert_ne!(mochi, biscuit);
        // Small, cosy things lean towards Mochi, and shiny, wild ones towards bold Biscuit.
        assert!(
            mochi["thyme_cutting"] > biscuit["thyme_cutting"],
            "{mochi:?} {biscuit:?}"
        );
        assert!(biscuit["golden_chanterelle"] > mochi["golden_chanterelle"]);
    }

    #[test]
    fn a_little_one_reaches_the_tucked_places_and_a_climber_or_a_close_pair_the_high_ones() {
        let cast = sample();
        let reaches = |party: Vec<Id>, close_pair: bool| {
            let (_, foray) = outing_with(&cast, party, close_pair, 5);
            let tucked = foray
                .items
                .iter()
                .filter(|i| i.slot.reach == Reach::Tucked)
                .count();
            let high = foray
                .items
                .iter()
                .filter(|i| i.slot.reach == Reach::High)
                .count();
            (tucked, high)
        };
        let tucked_slots = hedgerow::slots()
            .filter(|(_, s)| s.reach == Reach::Tucked)
            .count();
        let high_slots = hedgerow::slots()
            .filter(|(_, s)| s.reach == Reach::High)
            .count();
        assert_eq!(reaches(vec![named(&cast, "Pip")], false), (tucked_slots, 0));
        assert_eq!(reaches(vec![named(&cast, "Tansy")], false), (0, high_slots));
        assert_eq!(reaches(vec![named(&cast, "Mochi")], false), (0, 0));
        let pair = vec![named(&cast, "Mochi"), named(&cast, "Fig")];
        assert_eq!(reaches(pair.clone(), false), (0, 0));
        assert_eq!(
            reaches(pair, true),
            (0, high_slots),
            "a close pair bends the branch"
        );
        // Whoever reaches a place is the one who picks there.
        let (_, foray) = outing_with(
            &cast,
            vec![named(&cast, "Mochi"), named(&cast, "Pip")],
            false,
            5,
        );
        assert_eq!(foray.picker(Reach::Tucked), Some(named(&cast, "Pip")));
        assert_eq!(foray.picker(Reach::Within), Some(named(&cast, "Mochi")));
    }

    #[test]
    fn a_climber_goes_up_into_the_branches_and_comes_down_again() {
        let cast = sample();
        let tansy = named(&cast, "Tansy");
        let (mut ground, mut foray) = outing_with(&cast, vec![tansy], false, 6);
        run(&mut ground, &mut foray, &cast, 0.0, 6.0);
        let now = go_to(&mut ground, &mut foray, &cast, 6, 6.0);
        let high = foray
            .items
            .iter()
            .position(|item| item.slot.reach == Reach::High && item.patch == 6)
            .expect("high apples for a climber");
        foray.items[high].ripe_at = foray.gone() - 0.01;
        foray.pick(&mut ground, high, now);
        assert_eq!(foray.basket().len(), 1);
        let mut highest: f32 = 0.0;
        let mut later = now;
        while later < now + CLIMB_SECS + 0.5 {
            later += 1.0 / 30.0;
            ground.tick(&cast, later);
            foray.tick(&mut ground, later);
            highest = highest.max(foray.climb.map_or(0.0, |climb| {
                climb_height((later - climb.since) / CLIMB_SECS, false)
            }));
        }
        assert!(highest > LIFT * 0.9, "it never climbed");
        assert!(foray.climb.is_none(), "it stayed up the tree");
    }

    #[test]
    fn a_patient_paw_keeps_ripeness_a_little_longer() {
        let cast = sample();
        let mut kept = Vec::new();
        for impulsiveness in [0.0, 1.0] {
            let (mut ground, mut foray) =
                outing_with(&cast, vec![named(&cast, "Button")], false, 7);
            foray.characters[0].axes.impulsiveness = impulsiveness;
            run(&mut ground, &mut foray, &cast, 0.0, 6.0);
            let now = go_to(&mut ground, &mut foray, &cast, 0, 6.0);
            // Just over: a fifth of its window past ripe.
            let item = common_at(&mut foray, 0, 0.0);
            foray.items[item].ripe_at = foray.gone() - 0.12;
            assert_eq!(foray.stage_of(item), Some(Stage::Over));
            foray.pick(&mut ground, item, now);
            let events = foray.take_events();
            kept.push(foray.basket().len());
            if impulsiveness == 0.0 {
                assert!(
                    events
                        .iter()
                        .any(|event| matches!(event, Event::Saved { .. }))
                );
            }
        }
        assert_eq!(
            kept,
            [1, 0],
            "the patient one kept it and the hasty one didn't"
        );
    }

    #[test]
    fn one_that_looks_food_over_never_picks_too_early_and_now_and_then_eats_one() {
        let cast = sample();
        let biscuit = named(&cast, "Biscuit");
        let (mut ground, mut foray) = outing_with(&cast, vec![biscuit], false, 8);
        assert_eq!(foray.inspector, Some(biscuit));
        run(&mut ground, &mut foray, &cast, 0.0, 6.0);
        let mut now = go_to(&mut ground, &mut foray, &cast, 0, 6.0);
        let item = common_at(&mut foray, 0, 0.3);
        let light = foray.light;
        foray.pick(&mut ground, item, now);
        assert!(
            foray
                .take_events()
                .contains(&Event::LookedOver { who: biscuit, item })
        );
        assert!(foray.light > light - 0.5, "looking it over cost a picking");
        assert!(
            foray.inspected(item).is_some(),
            "it says how ripe a thing really is"
        );
        // With plenty in the basket already, a ripe one sometimes goes in a mouth instead.
        foray.basket = vec!["blackberries"];
        let mut eaten = 0;
        for _ in 0..30 {
            now += 1.0;
            let ripe = common_at(&mut foray, 0, -0.01);
            foray.light = foray.full * 0.6;
            foray.items[ripe].ripe_at = foray.gone() - 0.01;
            foray.basket.truncate(1);
            foray.pick(&mut ground, ripe, now);
            if foray
                .take_events()
                .iter()
                .any(|event| matches!(event, Event::Ate { .. }))
            {
                eaten += 1;
            }
        }
        assert!(eaten > 0 && eaten < 30, "ate {eaten} of 30");
        // Someone else never looks it over, and wastes the picking.
        let (mut ground, mut foray) = outing_with(&cast, vec![named(&cast, "Button")], false, 8);
        assert_eq!(foray.inspector, None);
        run(&mut ground, &mut foray, &cast, 0.0, 6.0);
        let now = go_to(&mut ground, &mut foray, &cast, 0, 6.0);
        let item = common_at(&mut foray, 0, 0.3);
        foray.pick(&mut ground, item, now);
        assert!(foray.take_events().contains(&Event::NotYet { item }));
        assert_eq!(foray.inspected(item), None);
    }

    #[test]
    fn someone_curious_spots_what_hides_and_says_when_something_rare_is_nearly_ripe() {
        let cast = sample();
        let hidden = |foray: &Foray| {
            foray
                .items
                .iter()
                .filter(|item| item.slot.hidden && !item.revealed)
                .count()
        };
        // Tansy is curious; Mochi isn't.
        let (mut ground, mut curious) = outing_with(&cast, vec![named(&cast, "Tansy")], false, 9);
        let (mut plain_ground, mut plain) =
            outing_with(&cast, vec![named(&cast, "Mochi")], false, 9);
        for foray in [&mut curious, &mut plain] {
            for (patch, slot) in hedgerow::slots().filter(|(_, slot)| slot.hidden) {
                if !foray.items.iter().any(|item| item.slot.at == slot.at) {
                    foray.items.push(Item {
                        slot: *slot,
                        patch,
                        find: slot.plant.find(),
                        ripe_at: 0.5,
                        window: 0.1,
                        revealed: false,
                        told: false,
                        fate: Fate::Growing,
                        gift: false,
                    });
                }
            }
        }
        assert!(hidden(&curious) > 0 && hidden(&plain) > 0);
        let events = run(&mut ground, &mut curious, &cast, 0.0, 40.0);
        run(&mut plain_ground, &mut plain, &cast, 0.0, 40.0);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::Spotted { .. }))
        );
        assert!(
            hidden(&curious) < hidden(&plain),
            "nothing spotted from the lane"
        );
        assert!(hidden(&plain) > 0);
        // Looking closely at a patch finds whatever hides there, whoever is looking.
        let patch = plain
            .items
            .iter()
            .find(|item| item.slot.hidden && !item.revealed)
            .map(|item| item.patch)
            .unwrap();
        go_to(&mut plain_ground, &mut plain, &cast, patch, 40.0);
        assert!(
            plain
                .items
                .iter()
                .filter(|item| item.patch == patch)
                .all(|item| item.revealed)
        );
        // Something rare turning ripe is told of, by the curious only.
        let rare = curious
            .items
            .iter()
            .position(|item| item.find.tier >= Tier::Rare)
            .unwrap_or_else(|| {
                let slot = hedgerow::slots()
                    .find(|(_, s)| s.plant == Plant::Chanterelle)
                    .unwrap();
                curious.items.push(Item {
                    slot: *slot.1,
                    patch: slot.0,
                    find: slot.1.plant.find(),
                    ripe_at: 0.0,
                    window: 0.06,
                    revealed: false,
                    told: false,
                    fate: Fate::Growing,
                    gift: false,
                });
                curious.items.len() - 1
            });
        curious.items[rare].ripe_at = curious.gone() + TURNING / 2.0;
        curious.items[rare].told = false;
        curious.tell_of_ripening(curious.gone());
        assert!(
            curious
                .take_events()
                .iter()
                .any(|event| matches!(event, Event::Ripening { item, .. } if *item == rare))
        );
    }

    #[test]
    fn a_sweetheart_leaves_the_ripe_ones_for_the_birds_and_the_birds_bring_something_back() {
        let cast = sample();
        let tansy = named(&cast, "Tansy");
        let (mut ground, mut foray) = outing_with(&cast, vec![tansy], false, 10);
        assert_eq!(foray.sweetheart, Some(tansy));
        run(&mut ground, &mut foray, &cast, 0.0, 6.0);
        let mut now = go_to(&mut ground, &mut foray, &cast, 0, 6.0);
        // Three ripe things left behind at the bramble.
        for _ in 0..3 {
            let item = common_at(&mut foray, 0, -0.01);
            foray.items[item].ripe_at = foray.gone() - 0.01;
            foray.items[item].fate = Fate::Growing;
            let _ = item;
            foray.items.push(foray.items[item].clone());
        }
        let ripe_here = foray
            .items
            .iter()
            .filter(|item| item.patch == 0 && item.revealed && item.growing())
            .filter(|item| item.stage(foray.gone()) == Stage::Ripe)
            .count() as u32;
        assert!(ripe_here >= THANKS);
        now = go_to(&mut ground, &mut foray, &cast, 2, now);
        let items_before = foray.items.len();
        let events = run(&mut ground, &mut foray, &cast, now, now + 3.0);
        assert!(foray.for_birds >= THANKS);
        let thanked = events.iter().find_map(|event| match event {
            Event::Thanked { who, item } => Some((*who, *item)),
            _ => None,
        });
        let (who, gift) = thanked.expect("no thanks from the birds");
        assert_eq!(who, tansy);
        assert_eq!(foray.items.len(), items_before + 1);
        assert_eq!(foray.items[gift].patch, 2, "brought to where the party is");
        assert_eq!(foray.stage_of(gift), Some(Stage::Ripe));
        assert_ne!(foray.items[gift].find.tier, Tier::Exceptional);
        // Only once a foray.
        foray.for_birds += 5;
        let events = run(&mut ground, &mut foray, &cast, now + 3.0, now + 6.0);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, Event::Thanked { .. }))
        );
        // Nobody sweet leaves anything for the birds.
        let (mut ground, mut foray) = outing_with(&cast, vec![named(&cast, "Mochi")], false, 10);
        run(&mut ground, &mut foray, &cast, 0.0, 6.0);
        let now = go_to(&mut ground, &mut foray, &cast, 0, 6.0);
        let item = common_at(&mut foray, 0, -0.01);
        foray.items[item].ripe_at = foray.gone() - 0.01;
        go_to(&mut ground, &mut foray, &cast, 2, now);
        assert!(
            foray.items[item].growing(),
            "left on the bush, still there to pick"
        );
        assert_eq!(foray.for_birds, 0);
    }

    #[test]
    fn the_basket_holds_six_and_something_can_be_put_back_to_make_room() {
        let cast = sample();
        let (mut ground, mut foray) = outing_with(&cast, vec![named(&cast, "Button")], false, 11);
        run(&mut ground, &mut foray, &cast, 0.0, 6.0);
        let mut now = go_to(&mut ground, &mut foray, &cast, 0, 6.0);
        foray.light = foray.full;
        for _ in 0..BASKET {
            let item = common_at(&mut foray, 0, -0.01);
            foray.items[item].ripe_at = foray.gone() - 0.01;
            foray.pick(&mut ground, item, now);
            now += 1.0;
        }
        assert_eq!(foray.basket().len(), BASKET);
        let item = common_at(&mut foray, 0, -0.01);
        foray.items[item].ripe_at = foray.gone() - 0.01;
        foray.take_events();
        foray.pick(&mut ground, item, now);
        assert!(foray.take_events().contains(&Event::BasketFull));
        assert_eq!(foray.basket().len(), BASKET);
        assert!(
            foray.items[item].growing(),
            "a full basket leaves it on the bush"
        );
        assert!(
            matches!(foray.phase(), Phase::At { .. }),
            "a full basket needn't go home"
        );
        let slot = foray.basket_slot_at(SCENE_WIDTH as f32 - 10.0, 9.0, SCENE_WIDTH);
        assert_eq!(slot, Some(BASKET - 1));
        foray.put_back(0);
        assert_eq!(foray.basket().len(), BASKET - 1);
        foray.pick(&mut ground, item, now + 1.0);
        assert_eq!(foray.basket().len(), BASKET);
    }

    #[test]
    fn walking_waiting_and_picking_spend_the_light_and_the_foray_ends_at_dusk() {
        let cast = sample();
        let (mut ground, mut foray) = outing_with(&cast, vec![named(&cast, "Fig")], false, 12);
        run(&mut ground, &mut foray, &cast, 0.0, 6.0);
        let start = foray.light;
        run(&mut ground, &mut foray, &cast, 6.0, 16.0);
        let waited = start - foray.light;
        assert!(
            (waited - LIGHT_PER_SEC * 10.0).abs() < 0.2,
            "waiting cost {waited}"
        );
        let before = foray.light;
        let now = go_to(&mut ground, &mut foray, &cast, 7, 16.0);
        let walked = before - foray.light;
        let seconds = now - 1.0 - 16.0;
        assert!(
            walked > LIGHT_PER_SEC * seconds + 3.0,
            "the walk cost only {walked}"
        );
        let events = run(&mut ground, &mut foray, &cast, now, now + 400.0);
        assert!(events.contains(&Event::Dusk));
        assert!(events.contains(&Event::Leaving(Ending::Dusk)), "{events:?}");
        assert_eq!(foray.phase(), Phase::Over);
    }

    #[test]
    fn with_reduced_motion_what_drops_cuts_to_the_ground_and_a_climb_is_two_cuts() {
        let mut snapshot = formiga_travel::sample::snapshot();
        snapshot.presentation.reduce_motion = true;
        let cast = Cast::new(snapshot).unwrap();
        let (_, mut foray) = outing_with(&cast, vec![cast.members[0].id], false, 13);
        assert!(foray.reduce_motion);
        let item = common_at(&mut foray, 0, 0.0);
        foray.items[item].fate = Fate::Fell { since: 5.0 };
        let floor = PATCHES[0].floor;
        let props = foray.produce(5.0);
        let (ax, ay) = produce::anchor(foray.items[item].slot.plant);
        let x = foray.items[item].slot.at.0 as i32 - ax;
        let dropped = props
            .iter()
            .find(|prop| prop.at.0 == x && (prop.base - floor).abs() < 0.5)
            .expect("it isn't drawn at all");
        assert_eq!(dropped.at.1 + ay, floor as i32, "it fell through the air");
        for step in 0..=20 {
            let height = climb_height(step as f32 / 20.0, true);
            assert!(
                height == 0.0 || height == LIFT,
                "caught halfway up at {height}"
            );
        }
        assert!(
            (1..20).any(|step| climb_height(step as f32 / 20.0, false) != 0.0
                && climb_height(step as f32 / 20.0, false) != LIFT)
        );
    }

    /// Plays a whole foray: `canny` reads ripeness and goes where things are ripening; otherwise
    /// it goes round the patches in turn and picks whatever it sees, ripe or not.
    fn play(canny: bool, seed: u64) -> (Vec<&'static str>, u32) {
        let cast = sample();
        let (mut ground, mut foray) = outing_with(&cast, vec![named(&cast, "Button")], false, seed);
        let (mut now, mut wasted, mut round) = (0.0, 0, 0);
        while foray.phase() != Phase::Over && now < 400.0 {
            now += 1.0 / 10.0;
            ground.tick(&cast, now);
            foray.tick(&mut ground, now);
            let events = foray.take_events();
            wasted += events
                .iter()
                .filter(|event| matches!(event, Event::NotYet { .. } | Event::Spoilt { .. }))
                .count() as u32;
            if canny {
                match foray.canny(&ground) {
                    Move::Pick(item) => foray.pick(&mut ground, item, now),
                    Move::PutBack(slot) => foray.put_back(slot),
                    Move::Go(patch) => foray.choose(&mut ground, patch, now),
                    Move::Wait => {}
                }
            } else if let Some(patch) = foray.at() {
                let seen = (0..foray.items.len()).find(|&item| {
                    foray.item(item).is_some_and(|(_, at, _)| at == patch)
                        && foray.stage_of(item).is_some()
                });
                match seen {
                    Some(item) if foray.basket().len() < BASKET => {
                        foray.pick(&mut ground, item, now)
                    }
                    _ => {
                        round += 1;
                        foray.choose(&mut ground, round % PATCHES.len(), now);
                    }
                }
            } else if foray.phase() == Phase::Choosing {
                foray.choose(&mut ground, 0, now);
            }
            if foray.basket().len() >= BASKET && !canny {
                foray.head_home(&mut ground, now);
            }
        }
        (foray.basket().to_vec(), wasted)
    }

    #[test]
    fn reading_the_ripeness_brings_home_better_things_than_picking_at_whatever_is_there() {
        let value = |basket: &[&str]| {
            basket
                .iter()
                .filter_map(|id| finds::find(id))
                .map(|find| find.tier as u32 * 3 + 1)
                .sum::<u32>()
        };
        let (mut canny_total, mut hasty_total, mut hasty_wasted) = (0, 0, 0);
        for seed in 0..6 {
            let (canny, wasted) = play(true, seed);
            assert_eq!(wasted, 0, "a canny hand never wastes a picking");
            assert_eq!(canny.len(), BASKET, "a canny hand fills the basket");
            let (hasty, wasted) = play(false, seed);
            canny_total += value(&canny);
            hasty_total += value(&hasty);
            hasty_wasted += wasted;
        }
        assert!(hasty_wasted > 0, "picking at random never went wrong");
        assert!(
            canny_total > hasty_total,
            "{canny_total} against {hasty_total}"
        );
    }

    #[test]
    fn every_companion_is_told_apart_as_a_forager() {
        let cast = sample();
        let said: BTreeSet<String> = cast
            .members
            .iter()
            .map(|m| forager(&Character::of(m)))
            .collect();
        assert!(said.len() >= 3, "{said:?}");
        let pip = cast.members.iter().find(|m| m.name == "Pip").unwrap();
        assert!(forager(&Character::of(pip)).starts_with("Small enough"));
    }
}
