//! A heap on the old track: planks, beams, stones, sacks, crates and old tins, left where they
//! fell long ago, each resting on whatever is under it, with things hidden among them. The good
//! things are underneath.
//!
//! A heap is built by dropping its things one after another, as they might have come down: the
//! ones on the ground first, then boards across them, then whatever else on top, each coming to
//! rest on the highest thing beneath it. So every heap is different, and reading one is a matter
//! of seeing what rests on what. Things lie hidden under anything, or inside a sack, a crate or a
//! tin, and in a hollow wherever a board bridges a gap.
//!
//! Lifting something nothing rests on is simple. Pulling something out from under others shifts
//! the heap: everything that rested on it comes tumbling down into its place, covering whatever
//! was under it, and anything fragile in the tumble (there is only the china teacup) cracks. Only
//! a gentle hand eases it out and lets the rest down softly. A beam or a boulder is heavy, and
//! wants someone strong or two together. Nothing that happens to the heap ever touches what is
//! already in the basket.

use crate::character::Character;
use crate::dice::Dice;
use crate::finds::{self, DROUGHT, Find, NOVELTY, SCAVENGED, Tier, drawn_to};

/// The things a heap is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stuff {
    Plank,
    Beam,
    Stone,
    Boulder,
    Sack,
    Crate,
    Tin,
}

impl Stuff {
    /// Too heavy for anyone but the strong, or two together.
    pub fn heavy(self) -> bool {
        matches!(self, Self::Beam | Self::Boulder)
    }

    /// Something can be inside it.
    pub fn container(self) -> bool {
        matches!(self, Self::Sack | Self::Crate | Self::Tin)
    }

    /// What the person calls it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Plank => "a plank",
            Self::Beam => "a beam",
            Self::Stone => "a stone",
            Self::Boulder => "a boulder",
            Self::Sack => "a sack",
            Self::Crate => "a crate",
            Self::Tin => "an old tin",
        }
    }

    /// How big it is, across and up, at random within its kind.
    fn size(self, dice: &mut Dice) -> (i32, i32) {
        let between = |dice: &mut Dice, low: i32, high: i32| {
            low + (dice.unit() * (high - low + 1) as f32) as i32 % (high - low + 1)
        };
        match self {
            Self::Plank => (between(dice, 30, 40), 4),
            Self::Beam => (between(dice, 38, 46), 6),
            Self::Stone => (between(dice, 11, 14), between(dice, 8, 9)),
            Self::Boulder => (between(dice, 17, 20), between(dice, 13, 15)),
            Self::Sack => (between(dice, 16, 19), between(dice, 12, 13)),
            Self::Crate => (between(dice, 17, 19), between(dice, 14, 16)),
            Self::Tin => (between(dice, 8, 9), between(dice, 8, 9)),
        }
    }
}

/// One thing in a heap.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub stuff: Stuff,
    /// Its left edge across the heap, and how high its underside is off the ground, in pixels.
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// What it rests on: nothing, if it is on the ground.
    pub rests_on: Vec<usize>,
    /// Lifted off the heap and set aside.
    pub lifted: bool,
    /// Someone has looked under it and into it.
    pub peeked: bool,
    /// For drawing it: so every plank has its own grain.
    pub salt: u32,
}

impl Item {
    pub fn top(&self) -> i32 {
        self.y + self.h
    }

    /// Whether it and `other` share any of the same columns.
    pub fn overlaps(&self, other: &Item) -> bool {
        self.x < other.x + other.w && other.x < self.x + self.w
    }

    fn covers(&self, column: i32) -> bool {
        column >= self.x && column < self.x + self.w
    }

    fn middle(&self) -> i32 {
        self.x + self.w / 2
    }
}

/// Where something lies hidden in a heap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// Under this item, on whatever it rests on.
    Under(usize),
    /// Inside this sack, crate or tin.
    Inside(usize),
    /// Out in the open, to be picked up.
    Open,
    /// Picked up.
    Taken,
}

/// Something hidden in a heap: a find, or a torn map.
#[derive(Clone, Copy, Debug)]
pub enum Hoard {
    Find(&'static Find),
    Map,
}

impl PartialEq for Hoard {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Find(a), Self::Find(b)) => a.id == b.id,
            (Self::Map, Self::Map) => true,
            _ => false,
        }
    }
}

impl Hoard {
    /// What breaks when the heap comes down on it, and what it becomes.
    fn breaks_into(self) -> Option<&'static Find> {
        match self {
            Self::Find(find) => CRACKS
                .iter()
                .find(|(whole, _)| *whole == find.id)
                .and_then(|(_, cracked)| finds::find(cracked)),
            Self::Map => None,
        }
    }

    pub fn fragile(self) -> bool {
        self.breaks_into().is_some()
    }

    /// Whether it fits in an old tin.
    fn small(self) -> bool {
        match self {
            Self::Find(find) => SMALL.contains(&find.id),
            Self::Map => true,
        }
    }
}

/// What cracks in a heap, and what it cracks into.
pub const CRACKS: [(&str, &str); 1] = [("china_teacup", "cracked_teacup")];
/// What fits in an old tin.
const SMALL: [&str; 2] = ["tin_soldier", "pocket_compass"];

/// Something hidden in a heap, and where.
#[derive(Clone, Debug, PartialEq)]
pub struct Hidden {
    pub what: Hoard,
    pub place: Place,
    /// It was whole once, and something came down on it.
    pub cracked: bool,
    /// Someone has seen what it is: peeked at it, or looked into what it is in.
    pub known: bool,
    /// Where across the heap it lies, and how high off the ground once it is out in the open.
    pub at: i32,
    pub lies: i32,
}

/// What a heap is made of, as it is built: things on the ground first, then boards across them,
/// then whatever lies on top. Each is a choice of stuff, and how many of them, at least and most.
#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub ground: (&'static [Stuff], (u32, u32)),
    pub across: (&'static [Stuff], (u32, u32)),
    pub top: (&'static [Stuff], (u32, u32)),
}

/// How a thing is lifted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum How {
    /// Lifted off; anything resting on it comes down.
    Plain,
    /// Eased out, and everything above it let down softly: nothing cracks, and whatever was under
    /// it is brought out first.
    Gentle,
    /// Yanked out, quickly: anything resting on it comes down, and anything breakable inside it is
    /// jolted and cracks.
    Yank,
}

/// What lifting something did to the heap.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shift {
    /// Everything that came down, and from how high its underside was.
    pub fell: Vec<(usize, i32)>,
    /// Hidden things that cracked.
    pub cracked: Vec<usize>,
    /// Hidden things now out in the open.
    pub revealed: Vec<usize>,
    /// Hidden things something came down on, under it now.
    pub buried: Vec<usize>,
}

/// Why something could not be done.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Already lifted.
    Gone,
    /// No gap under it to squeeze into.
    NoGap,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Heap {
    pub items: Vec<Item>,
    pub hidden: Vec<Hidden>,
    pub width: i32,
}

impl Heap {
    /// A heap of `width` built from `recipe`. Nothing is hidden in it yet (see `stock`).
    pub fn build(recipe: &Recipe, width: i32, dice: &mut Dice) -> Self {
        let mut heap = Self {
            items: Vec::new(),
            hidden: Vec::new(),
            width,
        };
        let count = |dice: &mut Dice, (low, high): (u32, u32)| {
            low + (dice.unit() * (high - low + 1) as f32) as u32 % (high - low + 1)
        };
        let choose = |dice: &mut Dice, from: &[Stuff]| {
            from[(dice.unit() * from.len() as f32) as usize % from.len()]
        };
        // On the ground, spread across the heap with room between them.
        let grounded = count(dice, recipe.ground.1).max(1) as i32;
        let slot = width / grounded;
        for index in 0..grounded {
            let stuff = choose(dice, recipe.ground.0);
            let (w, h) = stuff.size(dice);
            let w = w.min(slot - 2);
            let room = (slot - w).max(0);
            let x = index * slot + (dice.unit() * (room + 1) as f32) as i32 % (room + 1);
            heap.drop_in(stuff, x, (w, h), dice);
        }
        // Boards across: each from over one thing on the ground to over the next, if it reaches.
        let ground_things: Vec<usize> = (0..heap.items.len()).collect();
        for _ in 0..count(dice, recipe.across.1) {
            let stuff = choose(dice, recipe.across.0);
            let (w, h) = stuff.size(dice);
            let x = if ground_things.len() >= 2 {
                let first = (dice.unit() * (ground_things.len() - 1) as f32) as usize
                    % (ground_things.len() - 1);
                let (a, b) = (
                    &heap.items[ground_things[first]],
                    &heap.items[ground_things[first + 1]],
                );
                let span = b.middle() - a.middle();
                let middle = (a.middle() + b.middle()) / 2;
                let slack = (w - span).max(0) / 2;
                middle - w / 2 + (dice.unit() * (slack + 1) as f32) as i32 % (slack + 1) - slack / 2
            } else {
                (dice.unit() * (width - w + 1).max(1) as f32) as i32
            };
            heap.drop_in(stuff, x.clamp(-4, width - w + 4), (w, h), dice);
        }
        // Whatever else, on top, over the middle of the heap.
        let (left, right) = heap.span();
        for _ in 0..count(dice, recipe.top.1) {
            let stuff = choose(dice, recipe.top.0);
            let (w, h) = stuff.size(dice);
            let room = (right - left - w).max(0);
            let x = left + (dice.unit() * (room + 1) as f32) as i32 % (room + 1);
            heap.drop_in(stuff, x, (w, h), dice);
        }
        heap
    }

    /// Drops something in at `x`: it comes to rest on the highest things under it.
    fn drop_in(&mut self, stuff: Stuff, x: i32, (w, h): (i32, i32), dice: &mut Dice) {
        let mut item = Item {
            stuff,
            x,
            y: 0,
            w,
            h,
            rests_on: Vec::new(),
            lifted: false,
            peeked: false,
            salt: (dice.next_u64() & 0xffff) as u32,
        };
        let (y, under) = self.resting_place(&item, self.items.len());
        item.y = y;
        item.rests_on = under;
        self.items.push(item);
    }

    /// Where something across `item`'s columns would come to rest among the first `before`
    /// things in the heap: how high, and on what.
    fn resting_place(&self, item: &Item, before: usize) -> (i32, Vec<usize>) {
        let below: Vec<usize> = (0..before)
            .filter(|&index| !self.items[index].lifted && self.items[index].overlaps(item))
            .collect();
        let y = below
            .iter()
            .map(|&index| self.items[index].top())
            .max()
            .unwrap_or(0);
        let on = below
            .into_iter()
            .filter(|&index| self.items[index].top() == y && y > 0)
            .collect();
        (y, on)
    }

    /// The left and right ends of what is in the heap now.
    pub fn span(&self) -> (i32, i32) {
        let standing = self.items.iter().filter(|item| !item.lifted);
        let left = standing.clone().map(|item| item.x).min().unwrap_or(0);
        let right = standing
            .map(|item| item.x + item.w)
            .max()
            .unwrap_or(self.width);
        (left, right)
    }

    /// Everything still in the heap that rests directly on `index`.
    pub fn resting_on(&self, index: usize) -> Vec<usize> {
        (0..self.items.len())
            .filter(|&other| {
                !self.items[other].lifted && self.items[other].rests_on.contains(&index)
            })
            .collect()
    }

    /// Everything resting on `index`, and on those, all the way up.
    pub fn above(&self, index: usize) -> Vec<usize> {
        let mut above = Vec::new();
        let mut next = vec![index];
        while let Some(at) = next.pop() {
            for other in self.resting_on(at) {
                if !above.contains(&other) {
                    above.push(other);
                    next.push(other);
                }
            }
        }
        above.sort_unstable();
        above
    }

    /// The highest top of anything still in the heap across `column`, below `under` (and not
    /// `except`): where something lying there lies.
    fn floor(&self, column: i32, under: i32, except: Option<usize>) -> i32 {
        self.items
            .iter()
            .enumerate()
            .filter(|(index, item)| {
                Some(*index) != except && !item.lifted && item.covers(column) && item.top() <= under
            })
            .map(|(_, item)| item.top())
            .max()
            .unwrap_or(0)
    }

    /// Whether there is a gap under something, big enough for a little one to squeeze into: a
    /// board across two things, or anything overhanging what it rests on.
    pub fn gap(&self, index: usize) -> bool {
        let item = &self.items[index];
        !item.lifted
            && item.y > 0
            && (item.x..item.x + item.w)
                .filter(|&column| item.y - self.floor(column, item.y, Some(index)) >= 3)
                .count()
                >= 3
    }

    /// How high off the ground something hidden lies, where it lies.
    pub fn level(&self, hidden: usize) -> i32 {
        let what = &self.hidden[hidden];
        match what.place {
            Place::Under(index) => {
                let item = &self.items[index];
                self.floor(what.at, item.y, Some(index))
            }
            Place::Inside(index) => self.items[index].y + 1,
            Place::Open | Place::Taken => what.lies,
        }
    }

    /// How deep a place is: how many things rest on what it is under or in, all the way up.
    pub fn depth(&self, index: usize) -> usize {
        self.above(index).len()
    }

    /// The hidden things under or in something.
    pub fn hidden_at(&self, index: usize) -> Vec<usize> {
        (0..self.hidden.len())
            .filter(|&hidden| {
                matches!(self.hidden[hidden].place, Place::Under(at) | Place::Inside(at) if at == index)
            })
            .collect()
    }

    /// Lifts something off the heap, `how` it is lifted. Whatever was inside it comes out into the
    /// open; whatever was under it is open too, unless the heap shifts and something comes down
    /// on it. See `How`.
    pub fn lift(&mut self, index: usize, how: How) -> Result<Shift, Refusal> {
        if self.items.get(index).is_none_or(|item| item.lifted) {
            return Err(Refusal::Gone);
        }
        let mut shift = Shift::default();
        let lifted = self.items[index].clone();
        // Where whatever was under it lies, before anything moves.
        let under: Vec<(usize, i32)> = (0..self.hidden.len())
            .filter(|&hidden| self.hidden[hidden].place == Place::Under(index))
            .map(|hidden| (hidden, self.level(hidden)))
            .collect();
        self.items[index].lifted = true;
        // What was inside comes out, jolted if it was yanked.
        for hidden in 0..self.hidden.len() {
            if self.hidden[hidden].place == Place::Inside(index) {
                if how == How::Yank {
                    self.crack(hidden, &mut shift);
                }
                self.hidden[hidden].place = Place::Open;
                self.hidden[hidden].at = lifted.middle();
                self.hidden[hidden].lies = lifted.y;
                shift.revealed.push(hidden);
            }
        }
        for (hidden, lies) in under {
            self.hidden[hidden].lies = lies;
        }
        // Everything settles onto whatever is under it now, in the order it came down.
        let before: Vec<i32> = self.items.iter().map(|item| item.y).collect();
        for (at, was) in before.into_iter().enumerate() {
            if self.items[at].lifted {
                continue;
            }
            let (y, on) = self.resting_place(&self.items[at], at);
            self.items[at].y = y;
            self.items[at].rests_on = on;
            if y < was {
                shift.fell.push((at, was));
            }
        }
        let fell: Vec<usize> = shift.fell.iter().map(|(at, _)| *at).collect();
        // Whatever was under it: brought out first by a gentle hand, or come down on.
        for hidden in 0..self.hidden.len() {
            if self.hidden[hidden].place != Place::Under(index) {
                continue;
            }
            let column = self.hidden[hidden].at;
            let landed = fell
                .iter()
                .copied()
                .filter(|&at| self.items[at].covers(column))
                .min_by_key(|&at| self.items[at].y);
            match landed {
                Some(on) if how != How::Gentle => {
                    self.hidden[hidden].place = Place::Under(on);
                    self.crack(hidden, &mut shift);
                    shift.buried.push(hidden);
                }
                _ => {
                    self.hidden[hidden].place = Place::Open;
                    shift.revealed.push(hidden);
                }
            }
        }
        // Anything breakable in or under what came down is jolted.
        if how != How::Gentle {
            for hidden in 0..self.hidden.len() {
                let jolted = matches!(
                    self.hidden[hidden].place,
                    Place::Under(at) | Place::Inside(at) if fell.contains(&at)
                );
                if jolted {
                    self.crack(hidden, &mut shift);
                }
            }
        }
        // Whatever is out in the open drops onto whatever is under it now.
        for hidden in 0..self.hidden.len() {
            if self.hidden[hidden].place == Place::Open {
                let (at, lies) = (self.hidden[hidden].at, self.hidden[hidden].lies);
                self.hidden[hidden].lies = self.floor(at, lies, None);
            }
        }
        Ok(shift)
    }

    fn crack(&mut self, hidden: usize, shift: &mut Shift) {
        let what = &mut self.hidden[hidden];
        if let Some(cracked) = what.what.breaks_into() {
            what.what = Hoard::Find(cracked);
            what.cracked = true;
            shift.cracked.push(hidden);
        }
    }

    /// Looks under something and into it, without moving it: says what is there.
    pub fn peek(&mut self, index: usize) -> Result<Vec<usize>, Refusal> {
        if self.items.get(index).is_none_or(|item| item.lifted) {
            return Err(Refusal::Gone);
        }
        self.items[index].peeked = true;
        let there = self.hidden_at(index);
        for &hidden in &there {
            self.hidden[hidden].known = true;
        }
        Ok(there)
    }

    /// A little one squeezes into the gap under something and brings out whatever is there,
    /// whole, without moving anything.
    pub fn squeeze(&mut self, index: usize) -> Result<Vec<usize>, Refusal> {
        if self.items.get(index).is_none_or(|item| item.lifted) {
            return Err(Refusal::Gone);
        }
        if !self.gap(index) {
            return Err(Refusal::NoGap);
        }
        let under: Vec<usize> = (0..self.hidden.len())
            .filter(|&hidden| self.hidden[hidden].place == Place::Under(index))
            .collect();
        for &hidden in &under {
            self.hidden[hidden].place = Place::Taken;
            self.hidden[hidden].known = true;
        }
        Ok(under)
    }

    /// Picks something up out of the open.
    pub fn take(&mut self, hidden: usize) -> Option<Hoard> {
        let what = self.hidden.get_mut(hidden)?;
        (what.place == Place::Open).then(|| {
            what.place = Place::Taken;
            what.what
        })
    }

    /// Every place something could be hidden: under anything but a tin, and inside anything that
    /// holds things. Each with how deep it is, and whether it is under something heavy.
    fn places(&self) -> Vec<(Place, usize, bool)> {
        let mut places = Vec::new();
        for (index, item) in self.items.iter().enumerate() {
            if item.lifted {
                continue;
            }
            let depth = self.depth(index);
            if item.stuff != Stuff::Tin {
                places.push((Place::Under(index), depth, item.stuff.heavy()));
            }
            if item.stuff.container() {
                places.push((Place::Inside(index), depth, false));
            }
        }
        places
    }

    /// Hides something at a place, somewhere across what it is under.
    fn hide(&mut self, what: Hoard, place: Place, dice: &mut Dice) -> usize {
        let at = match place {
            Place::Under(index) | Place::Inside(index) => {
                let item = &self.items[index];
                let reach = (item.w / 3).max(1);
                item.middle() - reach / 2 + (dice.unit() * reach as f32) as i32 % reach.max(1)
            }
            Place::Open | Place::Taken => self.width / 2,
        };
        self.hidden.push(Hidden {
            what,
            place,
            cracked: false,
            known: false,
            at,
            lies: 0,
        });
        let hidden = self.hidden.len() - 1;
        self.hidden[hidden].lies = self.level(hidden);
        hidden
    }
}

/// How likely a place is to hold something: likelier the deeper it is, and inside things.
fn holds(place: Place, depth: usize) -> f32 {
    let inside = if matches!(place, Place::Inside(_)) {
        0.12
    } else {
        0.0
    };
    (0.2 + 0.09 * depth as f32 + inside).min(0.75)
}

/// How well a find of `tier` suits a place `depth` deep: the commoner things lie nearer the top,
/// the uncommon ones further in, and the rare ones only deep down.
fn suits(tier: Tier, depth: usize) -> f32 {
    let depth = depth as f32;
    match tier {
        Tier::Common => 1.0 / (1.0 + 0.3 * depth),
        Tier::Uncommon => 0.6 + 0.4 * depth,
        Tier::Rare if depth >= 2.0 => 0.5 * depth,
        Tier::Rare | Tier::Exceptional => 0.0,
    }
}

/// Whether a find can lie in a heap at all: anything but the exceptional things, which only ever
/// turn up in a chest.
pub fn in_heaps(find: &Find) -> bool {
    find.tier < Tier::Exceptional
}

/// Whether a find can lie in a chest at the end of a map: the rare and exceptional things.
pub fn in_chests(find: &Find) -> bool {
    find.tier >= Tier::Rare
}

/// What hides things in the heaps on one outing.
pub struct Stocking<'a> {
    pub party: &'a [&'a Character],
    pub found_before: &'a dyn Fn(&str) -> bool,
    /// Outings in a row that brought nothing new home.
    pub drought: u32,
    /// The chance a torn map is hidden somewhere among the heaps.
    pub map: f32,
    /// One that looks things over knows what is inside every sack, crate and tin.
    pub inspector: bool,
}

/// Hides finds in the heaps, and perhaps a torn map. Each place holds something with a chance
/// that grows with its depth, the rarer things deeper down and only once an outing; what the
/// party leans towards and what is still undiscovered come up more often. Under anything heavy
/// there is always something: an extra chance, for the strong or two together, at whatever turns
/// up elsewhere too. After a dry spell something new is certain, somewhere anyone can get at.
pub fn stock(heaps: &mut [Heap], stocking: &Stocking, dice: &mut Dice) {
    let weight = |find: &Find, depth: usize| {
        let novelty = if (stocking.found_before)(find.id) {
            1.0
        } else {
            NOVELTY
        };
        find.tier.weight()
            * drawn_to(find.leanings, stocking.party)
            * novelty
            * suits(find.tier, depth)
    };
    let mut stocked: Vec<&'static str> = Vec::new();
    // What is somewhere anyone can get at, not under anything heavy.
    let mut reachable: Vec<&'static str> = Vec::new();
    for heap in heaps.iter_mut() {
        for (place, depth, heavy) in heap.places() {
            if !heavy && !dice.chance(holds(place, depth)) {
                continue;
            }
            let pool: Vec<&'static Find> = SCAVENGED
                .iter()
                .filter(|find| in_heaps(find))
                .filter(|find| find.tier == Tier::Common || !stocked.contains(&find.id))
                .filter(|find| fits(Hoard::Find(find), place, heap))
                .collect();
            let weights: Vec<f32> = pool.iter().map(|find| weight(find, depth)).collect();
            if let Some(find) = dice.weighted(&weights).map(|index| pool[index]) {
                stocked.push(find.id);
                if !heavy {
                    reachable.push(find.id);
                }
                heap.hide(Hoard::Find(find), place, dice);
            }
        }
    }
    // A long run of nothing new: something new is certainly there, the least rare first, where
    // anyone can get at it.
    let undiscovered: Vec<&'static Find> = SCAVENGED
        .iter()
        .filter(|find| in_heaps(find) && !(stocking.found_before)(find.id))
        .filter(|find| !reachable.contains(&find.id))
        .collect();
    if stocking.drought >= DROUGHT
        && let Some(least) = undiscovered.iter().map(|find| find.tier).min()
    {
        let candidates: Vec<&'static Find> = undiscovered
            .into_iter()
            .filter(|find| find.tier == least)
            .collect();
        let weights: Vec<f32> = candidates
            .iter()
            .map(|find| find.tier.weight() * drawn_to(find.leanings, stocking.party))
            .collect();
        if let Some(new) = dice.weighted(&weights).map(|index| candidates[index]) {
            let mut places: Vec<(usize, Place, usize)> = Vec::new();
            for (at, heap) in heaps.iter().enumerate() {
                for (place, depth, heavy) in heap.places() {
                    if !heavy && fits(Hoard::Find(new), place, heap) {
                        places.push((at, place, depth));
                    }
                }
            }
            // The rare things deepest, the rest wherever.
            if new.tier >= Tier::Rare {
                let deepest = places.iter().map(|(_, _, depth)| *depth).max().unwrap_or(0);
                places.retain(|(_, _, depth)| *depth == deepest);
            }
            if !places.is_empty() {
                let (at, place, _) =
                    places[(dice.unit() * places.len() as f32) as usize % places.len()];
                heaps[at].hide(Hoard::Find(new), place, dice);
            }
        }
    }
    // Now and then, a torn map: in a tin or a crate if there is one, or under a board.
    if dice.chance(stocking.map) {
        let mut places: Vec<(usize, Place)> = Vec::new();
        for (at, heap) in heaps.iter().enumerate() {
            for (place, _, heavy) in heap.places() {
                let snug = match place {
                    Place::Inside(index) => heap.items[index].stuff != Stuff::Sack,
                    Place::Under(index) => heap.items[index].stuff == Stuff::Plank,
                    _ => false,
                };
                if snug && !heavy {
                    places.push((at, place));
                }
            }
        }
        if !places.is_empty() {
            let (at, place) = places[(dice.unit() * places.len() as f32) as usize % places.len()];
            heaps[at].hide(Hoard::Map, place, dice);
        }
    }
    if stocking.inspector {
        for heap in heaps.iter_mut() {
            for hidden in &mut heap.hidden {
                if matches!(hidden.place, Place::Inside(_)) {
                    hidden.known = true;
                }
            }
        }
    }
}

/// Whether something can be hidden at a place: only small things in a tin.
fn fits(what: Hoard, place: Place, heap: &Heap) -> bool {
    match place {
        Place::Inside(index) if heap.items[index].stuff == Stuff::Tin => what.small(),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::Cast;

    /// A heap laid out by hand: two stones on the ground, a plank across them, and a sack on the
    /// plank. Under the plank, between the stones, there is a gap.
    fn bridge() -> Heap {
        let item = |stuff: Stuff, x: i32, (w, h): (i32, i32)| Item {
            stuff,
            x,
            y: 0,
            w,
            h,
            rests_on: Vec::new(),
            lifted: false,
            peeked: false,
            salt: 1,
        };
        let mut heap = Heap {
            items: Vec::new(),
            hidden: Vec::new(),
            width: 60,
        };
        for (stuff, x, size) in [
            (Stuff::Stone, 4, (9, 6)),
            (Stuff::Stone, 30, (9, 6)),
            (Stuff::Plank, 2, (40, 3)),
            (Stuff::Sack, 14, (13, 9)),
        ] {
            let mut next = item(stuff, x, size);
            let (y, on) = heap.resting_place(&next, heap.items.len());
            next.y = y;
            next.rests_on = on;
            heap.items.push(next);
        }
        heap
    }

    fn teacup() -> Hoard {
        Hoard::Find(finds::find("china_teacup").unwrap())
    }

    fn soldier() -> Hoard {
        Hoard::Find(finds::find("tin_soldier").unwrap())
    }

    #[test]
    fn everything_rests_on_the_highest_things_under_it() {
        let heap = bridge();
        assert!(heap.items[0].rests_on.is_empty() && heap.items[0].y == 0);
        assert_eq!(
            heap.items[2].rests_on,
            vec![0, 1],
            "the plank bridges the stones"
        );
        assert_eq!(heap.items[2].y, 6);
        assert_eq!(heap.items[3].rests_on, vec![2], "the sack is on the plank");
        assert_eq!(heap.items[3].y, 9);
        assert_eq!(heap.resting_on(2), vec![3]);
        assert_eq!(
            heap.above(0),
            vec![2, 3],
            "lifting a stone would bring both down"
        );
        assert!(
            heap.gap(2),
            "there is room under the plank between the stones"
        );
        assert!(!heap.gap(0) && !heap.gap(3));
    }

    #[test]
    fn lifting_what_nothing_rests_on_moves_nothing_and_opens_what_was_under_it() {
        let mut heap = bridge();
        let mut dice = Dice::new(1);
        let soldier = heap.hide(soldier(), Place::Under(3), &mut dice);
        let shift = heap.lift(3, How::Plain).unwrap();
        assert!(shift.fell.is_empty());
        assert_eq!(shift.revealed, vec![soldier]);
        assert_eq!(heap.hidden[soldier].place, Place::Open);
        assert_eq!(heap.lift(3, How::Plain), Err(Refusal::Gone));
    }

    #[test]
    fn pulling_something_out_from_under_brings_down_what_rested_on_it_onto_what_was_under() {
        let mut heap = bridge();
        let mut dice = Dice::new(2);
        let soldier = heap.hide(soldier(), Place::Under(2), &mut dice);
        heap.hidden[soldier].at = 20;
        let shift = heap.lift(2, How::Plain).unwrap();
        assert_eq!(shift.fell, vec![(3, 9)], "the sack came down");
        assert!(heap.items[3].y < 9);
        assert_eq!(
            heap.hidden[soldier].place,
            Place::Under(3),
            "and landed on what was under the plank"
        );
        assert_eq!(shift.buried, vec![soldier]);
        assert!(shift.revealed.is_empty());
        // Lifting the sack in turn opens it up.
        let shift = heap.lift(3, How::Plain).unwrap();
        assert_eq!(shift.revealed, vec![soldier]);
    }

    #[test]
    fn something_fragile_cracks_when_the_heap_comes_down_on_it_but_not_when_eased_out() {
        let lifted = |how: How| {
            let mut heap = bridge();
            let mut dice = Dice::new(3);
            let cup = heap.hide(teacup(), Place::Under(2), &mut dice);
            heap.hidden[cup].at = 20;
            let shift = heap.lift(2, how).unwrap();
            (heap.hidden[cup].clone(), shift)
        };
        let (cup, shift) = lifted(How::Plain);
        assert!(cup.cracked);
        assert_eq!(
            cup.what,
            Hoard::Find(finds::find("cracked_teacup").unwrap())
        );
        assert_eq!(shift.cracked.len(), 1);
        let (cup, shift) = lifted(How::Gentle);
        assert!(!cup.cracked, "eased out, the teacup is whole");
        assert_eq!(
            cup.place,
            Place::Open,
            "and brought out before the sack came down"
        );
        assert_eq!(shift.fell.len(), 1, "the sack still settles, softly");
        // From the top down, nothing ever comes down on it.
        let mut heap = bridge();
        let mut dice = Dice::new(4);
        let cup = heap.hide(teacup(), Place::Under(2), &mut dice);
        heap.lift(3, How::Plain).unwrap();
        let shift = heap.lift(2, How::Plain).unwrap();
        assert!(shift.cracked.is_empty() && !heap.hidden[cup].cracked);
        assert_eq!(heap.hidden[cup].place, Place::Open);
    }

    #[test]
    fn whatever_is_in_something_that_comes_down_is_jolted_and_a_yank_jolts_what_is_inside() {
        let mut heap = bridge();
        let mut dice = Dice::new(5);
        let in_sack = heap.hide(teacup(), Place::Inside(3), &mut dice);
        heap.lift(2, How::Plain).unwrap();
        assert!(
            heap.hidden[in_sack].cracked,
            "the sack fell, and the cup in it cracked"
        );
        assert_eq!(
            heap.hidden[in_sack].place,
            Place::Inside(3),
            "still in the sack"
        );
        let mut heap = bridge();
        let in_sack = heap.hide(teacup(), Place::Inside(3), &mut dice);
        heap.lift(3, How::Plain).unwrap();
        assert!(
            !heap.hidden[in_sack].cracked,
            "lifted off and opened, it is whole"
        );
        let mut heap = bridge();
        let in_sack = heap.hide(teacup(), Place::Inside(3), &mut dice);
        heap.lift(3, How::Yank).unwrap();
        assert!(heap.hidden[in_sack].cracked, "yanked, it cracked");
        assert_eq!(heap.hidden[in_sack].place, Place::Open);
    }

    #[test]
    fn only_what_breaks_cracks_and_a_map_never_does() {
        let mut heap = bridge();
        let mut dice = Dice::new(6);
        let map = heap.hide(Hoard::Map, Place::Inside(3), &mut dice);
        let toy = heap.hide(soldier(), Place::Inside(3), &mut dice);
        heap.lift(2, How::Yank).unwrap();
        heap.lift(3, How::Yank).unwrap();
        assert!(!heap.hidden[map].cracked && !heap.hidden[toy].cracked);
        assert_eq!(heap.hidden[map].what, Hoard::Map);
    }

    #[test]
    fn a_peek_shows_what_is_there_without_moving_anything() {
        let mut heap = bridge();
        let mut dice = Dice::new(7);
        let toy = heap.hide(soldier(), Place::Under(2), &mut dice);
        let before = heap.items.clone();
        assert_eq!(heap.peek(2), Ok(vec![toy]));
        assert!(heap.hidden[toy].known);
        assert_eq!(heap.hidden[toy].place, Place::Under(2));
        assert!(
            heap.items
                .iter()
                .zip(&before)
                .all(|(a, b)| a.y == b.y && !a.lifted)
        );
        assert_eq!(heap.peek(0), Ok(vec![]));
        assert!(heap.items[0].peeked);
    }

    #[test]
    fn a_little_one_squeezes_into_a_gap_and_brings_things_out_whole() {
        let mut heap = bridge();
        let mut dice = Dice::new(8);
        let cup = heap.hide(teacup(), Place::Under(2), &mut dice);
        let before = heap.items.clone();
        assert_eq!(heap.squeeze(2), Ok(vec![cup]));
        assert_eq!(heap.hidden[cup].place, Place::Taken);
        assert!(!heap.hidden[cup].cracked);
        assert_eq!(heap.items, before, "nothing in the heap moved");
        assert_eq!(
            heap.squeeze(0),
            Err(Refusal::NoGap),
            "a stone on the ground"
        );
    }

    #[test]
    fn heavy_things_are_beams_and_boulders_and_only_containers_hold_things() {
        for stuff in [
            Stuff::Plank,
            Stuff::Beam,
            Stuff::Stone,
            Stuff::Boulder,
            Stuff::Sack,
            Stuff::Crate,
            Stuff::Tin,
        ] {
            assert_eq!(stuff.heavy(), matches!(stuff, Stuff::Beam | Stuff::Boulder));
            assert_eq!(
                stuff.container(),
                matches!(stuff, Stuff::Sack | Stuff::Crate | Stuff::Tin)
            );
        }
    }

    const RECIPE: Recipe = Recipe {
        ground: (
            &[Stuff::Crate, Stuff::Stone, Stuff::Sack, Stuff::Boulder],
            (2, 3),
        ),
        across: (&[Stuff::Plank, Stuff::Beam], (1, 2)),
        top: (
            &[Stuff::Tin, Stuff::Sack, Stuff::Stone, Stuff::Plank],
            (1, 3),
        ),
    };

    #[test]
    fn a_built_heap_is_stacked_with_nothing_floating_or_overlapping() {
        for seed in 0..200 {
            let heap = Heap::build(&RECIPE, 60, &mut Dice::new(seed));
            assert!(
                heap.items.len() >= 4,
                "seed {seed}: only {}",
                heap.items.len()
            );
            for (index, item) in heap.items.iter().enumerate() {
                let (y, on) = heap.resting_place(item, index);
                assert_eq!(
                    (item.y, &item.rests_on),
                    (y, &on),
                    "seed {seed}, item {index}"
                );
                for (other, there) in heap.items.iter().enumerate() {
                    if other != index && item.overlaps(there) {
                        let apart = item.top() <= there.y || there.top() <= item.y;
                        assert!(apart, "seed {seed}: {index} and {other} overlap");
                    }
                }
            }
            let (left, right) = heap.span();
            assert!(left >= -4 && right <= 64, "seed {seed}: {left}..{right}");
        }
    }

    fn party() -> Vec<Character> {
        let cast = Cast::new(formiga_travel::sample::snapshot()).unwrap();
        cast.members.iter().map(Character::of).collect()
    }

    #[test]
    fn the_good_things_are_underneath_and_the_exceptional_never_in_a_heap() {
        let characters = party();
        let party: Vec<&Character> = characters.iter().collect();
        let mut dice = Dice::new(9);
        let (mut rare_depth, mut rares, mut common_depth, mut commons) = (0, 0, 0, 0);
        for _ in 0..300 {
            let mut heaps: Vec<Heap> = (0..3)
                .map(|_| Heap::build(&RECIPE, 60, &mut dice))
                .collect();
            let stocking = Stocking {
                party: &party,
                found_before: &|_| false,
                drought: 0,
                map: 0.0,
                inspector: false,
            };
            stock(&mut heaps, &stocking, &mut dice);
            for heap in &heaps {
                for hidden in &heap.hidden {
                    let (Hoard::Find(find), Place::Under(at) | Place::Inside(at)) =
                        (hidden.what, hidden.place)
                    else {
                        continue;
                    };
                    assert_ne!(find.tier, Tier::Exceptional, "{} in a heap", find.id);
                    assert!(finds::is_scavenged(find.id));
                    if heap.items[at].stuff.heavy() {
                        continue;
                    }
                    match find.tier {
                        Tier::Rare => {
                            rares += 1;
                            rare_depth += heap.depth(at);
                        }
                        Tier::Common => {
                            commons += 1;
                            common_depth += heap.depth(at);
                        }
                        _ => {}
                    }
                }
            }
        }
        assert!(rares > 0 && commons > 0);
        assert!(
            rare_depth as f32 / rares as f32 > common_depth as f32 / commons as f32 + 1.0,
            "rare things lie no deeper than common ones"
        );
    }

    #[test]
    fn under_anything_heavy_there_is_always_something_and_a_tin_holds_only_small_things() {
        let characters = party();
        let party: Vec<&Character> = characters.iter().collect();
        let mut dice = Dice::new(10);
        for _ in 0..300 {
            let mut heaps: Vec<Heap> = (0..3)
                .map(|_| Heap::build(&RECIPE, 60, &mut dice))
                .collect();
            let stocking = Stocking {
                party: &party,
                found_before: &|_| false,
                drought: 0,
                map: 0.5,
                inspector: true,
            };
            stock(&mut heaps, &stocking, &mut dice);
            for heap in &heaps {
                for (index, item) in heap.items.iter().enumerate() {
                    if item.stuff.heavy() {
                        assert!(
                            heap.hidden
                                .iter()
                                .any(|hidden| hidden.place == Place::Under(index)),
                            "nothing under a heavy {:?}",
                            item.stuff
                        );
                    }
                }
                for hidden in &heap.hidden {
                    if let Place::Inside(at) = hidden.place {
                        assert!(
                            hidden.known,
                            "one that looks things over knows what is inside"
                        );
                        if heap.items[at].stuff == Stuff::Tin {
                            assert!(hidden.what.small(), "{:?} in a tin", hidden.what);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn after_a_dry_spell_something_new_is_certainly_somewhere_anyone_can_get_at() {
        let characters = party();
        let party: Vec<&Character> = characters.iter().collect();
        let mut dice = Dice::new(11);
        // Everything found but the compass, the rarest thing a heap holds.
        let found = |id: &str| id != "pocket_compass";
        let mut certain = 0;
        for _ in 0..100 {
            let mut heaps: Vec<Heap> = (0..3)
                .map(|_| Heap::build(&RECIPE, 60, &mut dice))
                .collect();
            let stocking = Stocking {
                party: &party,
                found_before: &found,
                drought: DROUGHT,
                map: 0.0,
                inspector: false,
            };
            stock(&mut heaps, &stocking, &mut dice);
            let there = heaps.iter().any(|heap| {
                heap.hidden.iter().any(|hidden| {
                    hidden.what == Hoard::Find(finds::find("pocket_compass").unwrap())
                        && matches!(hidden.place, Place::Under(at) | Place::Inside(at)
                            if !heap.items[at].stuff.heavy())
                })
            });
            certain += usize::from(there);
        }
        assert_eq!(certain, 100);
    }
}
