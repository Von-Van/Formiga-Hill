//! The old track in the window: setting off to scavenge or to follow a map, choosing who does
//! what at a heap, the way at a fork and where to dig, and bringing everything home.

use super::HillApp;
use crate::cast::{Cast, Id};
use crate::character::Character;
use crate::finds::{self, SCAVENGED, Tier};
use crate::memories::ColonyMemories;
use crate::playground::Playground;
use crate::track::crew::{self, Hand};
use crate::track::heap::{How, Stuff};
use crate::track::scavenging::{BASKET, Ending, Event, Outset, Phase, Scavenge};
use crate::track::treasure::{self, Hunt};
use crate::track::{self, routes};
use eframe::egui;
use formiga_art::Canvas;
use formiga_travel::Band;

impl HillApp {
    /// Readies the old track, with nobody there, its heaps lying as they lie and the Hilltop
    /// through the gap as it stands now.
    pub(super) fn open_track(&mut self, now: f32) {
        let hilltop = self.hilltop_standing();
        match &mut self.woods.track {
            Some(track) => track::show_hilltop(track, &hilltop),
            None => {
                let mut ground = track::open(&self.arrival.cast, &[], now, &hilltop);
                ground.set_props(track::at_rest());
                ground.set_daylight(self.daylight);
                self.woods.track = Some(ground);
            }
        }
    }

    /// Whether the two going are close friends.
    fn close_pair(&self, party: &[Id]) -> bool {
        match party {
            [a, b] => self
                .arrival
                .cast
                .bond(*a, *b)
                .is_some_and(|bond| bond.warmth >= Band::High),
            _ => false,
        }
    }

    /// A seed for an outing, from the colony and how many outings like it there have been.
    fn seed(&self, count: u32, salt: u64) -> u64 {
        self.arrival
            .cast
            .snapshot
            .colony_id
            .bytes()
            .fold(u64::from(count) << 32 | salt, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
            })
    }

    /// Sets off up the old track to scavenge, with whoever was chosen; the first leads.
    pub(super) fn set_off_scavenging(&mut self, now: f32) {
        let party = self.woods.party.clone();
        if party.is_empty() {
            return;
        }
        let close_pair = self.close_pair(&party);
        let colony = self.memories.colony();
        let seed = self.seed(colony.scavenges, 0x5ca);
        let mut ground = track::open(&self.arrival.cast, &party, now, &colony.hilltop);
        ground.set_daylight(self.daylight);
        let influence = crate::woods::influence(&colony.hilltop);
        let note = influence.notes.first().copied();
        let outset = Outset {
            party,
            drought: colony.scavenge_drought,
            close_pair,
            influence,
            maps_held: colony.maps.len(),
            seed,
        };
        let found = |id: &str| colony.finds.contains_key(id);
        let mut scavenge = Scavenge::new(&mut ground, outset, found, now);
        scavenge.set_hour_dark(self.daylight.darkness());
        self.woods.scavenge = Some((ground, scavenge));
        let line = note.unwrap_or(
            "The good things are underneath. Click a heap to go to it, then choose who does what.",
        );
        self.notice = Some((line.to_owned(), now));
    }

    /// Sets off to follow the chosen map, with whoever was chosen; the first leads.
    pub(super) fn set_off_treasure(&mut self, now: f32) {
        let party = self.woods.party.clone();
        let colony = self.memories.colony();
        let Some(map) = colony.maps.get(self.woods.map).or(colony.maps.first()) else {
            return;
        };
        if party.is_empty() {
            return;
        }
        let close_pair = self.close_pair(&party);
        let seed = self.seed(colony.treasure_hunts, 0x7ea);
        let mut ground = track::hunt_ground(&self.arrival.cast, &party, now);
        ground.set_daylight(self.daylight);
        let influence = crate::woods::influence(&colony.hilltop);
        let outset = treasure::Outset {
            party,
            close_pair,
            influence,
            map: map.seed,
            drought: colony.chest_drought,
            seed,
        };
        let found = |id: &str| colony.finds.contains_key(id);
        let mut hunt = Hunt::new(&mut ground, outset, found, now);
        hunt.set_hour_dark(self.daylight.darkness());
        self.woods.treasure = Some((ground, hunt));
        self.notice = Some((
            "Off along the old track with the map. Match each line to the ways, and click the way \
             to take."
                .to_owned(),
            now,
        ));
    }

    /// Plays the scavenge on.
    pub(super) fn tick_scavenging(&mut self, now: f32) {
        let cast = &self.arrival.cast;
        let Some((ground, scavenge)) = &mut self.woods.scavenge else {
            return;
        };
        ground.tick(cast, now);
        scavenge.tick(ground, now);
        for event in scavenge.take_events() {
            if let Some(cue) = super::sound::scavenged(&event) {
                self.sound.play(cue);
            }
            if let Some(line) = heap_line(cast, event) {
                self.notice = Some((line, now));
            }
        }
        if scavenge.phase() == Phase::Over {
            self.finish_scavenging(now);
        }
    }

    /// Brings the basket home from the old track: into the journal and the satchel, with any maps
    /// found kept for another day.
    pub(super) fn finish_scavenging(&mut self, now: f32) {
        let Some((_, scavenge)) = self.woods.scavenge.take() else {
            return;
        };
        let basket = scavenge.basket().to_vec();
        let maps = scavenge.maps().to_vec();
        let new = self
            .memories
            .back_from_scavenging(scavenge.party(), &basket, &maps);
        let mut line = match (basket.len(), new.len()) {
            (0, _) => "Home from the old track, the basket empty this time.".to_owned(),
            (count, 0) => format!(
                "Home from the old track with {count} find{}. They're in the satchel, for the \
                 Hilltop.",
                plural(count)
            ),
            (count, fresh) => format!(
                "Home from the old track with {count} find{}, {fresh} new to the journal. \
                 They're in the satchel, for the Hilltop.",
                plural(count)
            ),
        };
        match maps.len() {
            0 => {}
            1 => line.push_str(" And a torn map, to follow another day."),
            count => line.push_str(&format!(" And {count} torn maps, to follow another day.")),
        }
        self.notice = Some((line, now));
    }

    /// Plays the treasure hunt on.
    pub(super) fn tick_treasure(&mut self, now: f32) {
        let cast = &self.arrival.cast;
        let Some((ground, hunt)) = &mut self.woods.treasure else {
            return;
        };
        ground.tick(cast, now);
        hunt.tick(ground, now);
        let name = |id: Id| cast.member(id).map_or("Someone", |m| m.name.as_str());
        let mut treasure_found: Option<&'static str> = None;
        for event in hunt.take_events() {
            if let Some(cue) = super::sound::treasure_heard(&event) {
                self.sound.play(cue);
            }
            let route = hunt.route();
            let read = hunt.reads_faded();
            let way_name = |way: usize| {
                hunt.way_dir(way)
                    .map_or("that way", routes::Dir::name)
                    .to_owned()
            };
            let line = match event {
                treasure::Event::Fork { fork } => route
                    .forks
                    .get(fork)
                    .map(|fork| format!("The map says: {}", fork.line(read))),
                treasure::Event::Read { who } => Some(format!(
                    "{} can make out the faded words: {}",
                    name(who),
                    hunt.line().unwrap_or_default()
                )),
                treasure::Event::Spotted { who, way } => {
                    let landmark = hunt
                        .fork()
                        .and_then(|fork| route.forks[fork].sought())
                        .map_or("something", |(landmark, _)| landmark.name());
                    Some(format!(
                        "{} spots {landmark}, far off down {}!",
                        name(who),
                        way_name(way)
                    ))
                }
                treasure::Event::Hunch { who, way } => Some(match hunt.phase() {
                    treasure::Phase::AtDig => {
                        let by = route.dig.spots[way].feature.name();
                        format!("{} has a hunch about the place by {by}.", name(who))
                    }
                    _ => format!("{} has a hunch it's {}.", name(who), way_name(way)),
                }),
                treasure::Event::Doubt { who, way } => Some(format!(
                    "{} isn't sure about {}. Click it again to go anyway.",
                    name(who),
                    way_name(way)
                )),
                treasure::Event::Tried { .. } => {
                    Some("You've been that way: it only led to a heap.".to_owned())
                }
                treasure::Event::Nook => Some(
                    "A dead end, but there's a heap here. Scavenge it, or go back to the fork."
                        .to_owned(),
                ),
                treasure::Event::Back => Some("Back at the fork.".to_owned()),
                treasure::Event::Heap(event) => heap_line(cast, event),
                treasure::Event::Milestone => Some(format!(
                    "The old milestone! The map says: {}",
                    route.dig.line(read)
                )),
                treasure::Event::Nothing { .. } => {
                    Some("Only roots and stones down there.".to_owned())
                }
                treasure::Event::Chest => Some("There's something down there\u{2026}".to_owned()),
                treasure::Event::Treasure { find } => {
                    treasure_found = Some(find);
                    Some(format!("A chest! And in it: {}!", lower(find_name(find))))
                }
                treasure::Event::Also { find } => Some(match treasure_found {
                    Some(first) => format!(
                        "A chest! And in it: {}, and {}.",
                        lower(find_name(first)),
                        lower(find_name(find))
                    ),
                    None => format!("And {} too.", lower(find_name(find))),
                }),
                treasure::Event::Leaving(treasure::Ending::Found) => None,
                treasure::Event::Leaving(treasure::Ending::Dusk) => {
                    Some("The light's gone. The map will keep for another day.".to_owned())
                }
                treasure::Event::Leaving(treasure::Ending::Chose) => {
                    Some("Heading home. The map will keep.".to_owned())
                }
            };
            if let Some(line) = line {
                self.notice = Some((line, now));
            }
        }
        if hunt.phase() == treasure::Phase::Over {
            self.finish_treasure(now);
        }
    }

    /// Brings home whatever the hunt found, the chest too if it was dug up.
    pub(super) fn finish_treasure(&mut self, now: f32) {
        let Some((_, hunt)) = self.woods.treasure.take() else {
            return;
        };
        let new = self.memories.back_from_a_treasure_hunt(
            hunt.party(),
            hunt.map(),
            hunt.basket(),
            hunt.chest(),
            hunt.maps(),
        );
        let line = if hunt.dug_up() {
            let treasure = hunt.chest().first().copied().map_or("something", find_name);
            let fresh = if new.is_empty() {
                String::new()
            } else {
                format!(", {} new to the journal", new.len())
            };
            format!(
                "Home with the chest: {}{fresh}. It's in the satchel, for the Hilltop.",
                lower(treasure)
            )
        } else if hunt.basket().is_empty() {
            "Home again. The map is kept, and the way will be the same.".to_owned()
        } else {
            format!(
                "Home with {} find{} from the heaps. The map is kept, and the way will be the same.",
                hunt.basket().len(),
                plural(hunt.basket().len())
            )
        };
        self.notice = Some((line, now));
    }

    pub(super) fn compose_scavenge(&mut self, now: f32) -> Option<Canvas> {
        let pointer = self.pointer;
        let (ground, scavenge) = self.woods.scavenge.as_mut()?;
        ground.set_fliers(scavenge.props(now));
        let mut scene = ground.compose(now);
        let hovered = pointer.and_then(|(x, y)| scavenge.item_at(x, y));
        scavenge.draw(&mut scene, hovered, now);
        Some(scene)
    }

    pub(super) fn compose_treasure(&mut self, now: f32) -> Option<Canvas> {
        let pointer = self.pointer;
        let (ground, hunt) = self.woods.treasure.as_mut()?;
        ground.set_fliers(hunt.props(now));
        let mut scene = ground.compose(now);
        let hovered = pointer.and_then(|(x, y)| hunt.nook().and_then(|nook| nook.item_at(x, y)));
        hunt.draw(&mut scene, hovered, now);
        Some(scene)
    }

    /// A click on the old track: put something back from the basket, pick up something lying in
    /// the open, do the chosen thing to something in the heap the party is at, or go to a heap.
    pub(super) fn scavenging_click(&mut self, pointer: Option<(f32, f32)>, now: f32) {
        let Some((ground, scavenge)) = &mut self.woods.scavenge else {
            return;
        };
        heap_click(ground, scavenge, pointer, now, true);
    }

    /// A click on a treasure hunt: a way at a fork, a place to dig, or the nook's heap.
    pub(super) fn treasure_click(&mut self, pointer: Option<(f32, f32)>, now: f32) {
        let Some((ground, hunt)) = &mut self.woods.treasure else {
            return;
        };
        let Some((x, y)) = pointer else {
            return;
        };
        if let Some(nook) = hunt.nook_mut() {
            heap_click(ground, nook, pointer, now, false);
            return;
        }
        if let Some(way) = hunt.way_at(x, y) {
            hunt.choose(ground, way, now);
        } else if let Some(spot) = hunt.spot_at(x, y) {
            hunt.dig(ground, spot, now);
            self.sound.play(crate::audio::Cue::Rummage);
        }
    }

    pub(super) fn scavenging_hover(
        &self,
        pointer: Option<(f32, f32)>,
    ) -> Option<(String, (f32, f32))> {
        let (_, scavenge) = self.woods.scavenge.as_ref()?;
        heap_hover(scavenge, pointer, true)
    }

    pub(super) fn treasure_hover(
        &self,
        pointer: Option<(f32, f32)>,
    ) -> Option<(String, (f32, f32))> {
        let (_, hunt) = self.woods.treasure.as_ref()?;
        if let Some(nook) = hunt.nook() {
            return heap_hover(nook, pointer, false);
        }
        let (x, y) = pointer?;
        if let Some(way) = hunt.way_at(x, y) {
            let label = hunt.describe_way(way)?;
            return Some((label, (x, y - 14.0)));
        }
        let spot = hunt.spot_at(x, y)?;
        let at = hunt.route().dig.spots[spot].at;
        Some((hunt.describe_spot(spot)?, (at.0, at.1 - 16.0)))
    }

    pub(super) fn scavenging_bar(&mut self, ui: &mut egui::Ui, now: f32) {
        let cast = &self.arrival.cast;
        let Some((ground, scavenge)) = &mut self.woods.scavenge else {
            return;
        };
        light_and_basket(ui, scavenge.light_left(), scavenge.basket().len(), 0);
        if !scavenge.maps().is_empty() {
            ui.label(format!(
                "{} torn map{}",
                scavenge.maps().len(),
                plural(scavenge.maps().len())
            ));
        }
        hands_bar(ui, cast, scavenge);
        let hint = match scavenge.phase() {
            Phase::Choosing | Phase::Going { .. } => "Click a heap to go to it.",
            Phase::At { .. } if scavenge.basket().len() >= BASKET => {
                "Full! Click a basket slot to put something back."
            }
            Phase::At { .. } => "Click something in the heap.",
            _ => "",
        };
        if !hint.is_empty() {
            ui.label(egui::RichText::new(hint).strong());
        }
        let leaving = matches!(scavenge.phase(), Phase::Leaving { .. } | Phase::Over);
        if ui
            .add_enabled(!leaving, egui::Button::new("Head home"))
            .clicked()
        {
            scavenge.head_home(ground, now);
        }
    }

    pub(super) fn treasure_bar(&mut self, ui: &mut egui::Ui, now: f32) {
        let cast = &self.arrival.cast;
        let Some((ground, hunt)) = &mut self.woods.treasure else {
            return;
        };
        if let Some(nook) = hunt.nook_mut() {
            light_and_basket(ui, nook.light_left(), nook.basket().len(), 0);
            hands_bar(ui, cast, nook);
            if ui.button("Back to the fork").clicked() {
                hunt.back(ground, now);
            }
        } else {
            light_and_basket(
                ui,
                hunt.light_left(),
                hunt.basket().len(),
                hunt.chest().len(),
            );
            let read = hunt.reads_faded();
            let lines = hunt.route().lines(read);
            if let Some(line) = hunt.line() {
                let whole = lines.join("\n");
                ui.label(egui::RichText::new(format!("The map: {line}")).strong())
                    .on_hover_text(format!("The whole map:\n{whole}"));
            }
            let hint = match hunt.phase() {
                treasure::Phase::Reading { .. } => "Click the way to take.",
                treasure::Phase::AtDig => "Click where to dig.",
                _ => "",
            };
            if !hint.is_empty() {
                ui.label(egui::RichText::new(hint).italics());
            }
        }
        let leaving = matches!(
            hunt.phase(),
            treasure::Phase::Leaving { .. } | treasure::Phase::Over
        );
        if ui
            .add_enabled(!leaving, egui::Button::new("Head home"))
            .clicked()
        {
            hunt.head_home(ground, now);
        }
    }

    /// The old track's choices on the Woods bar, beside the others: scavenging, and following a
    /// map once there is one, with which map.
    pub(super) fn track_activities(&mut self, ui: &mut egui::Ui) {
        use super::rummaging::Activity;
        ui.selectable_value(&mut self.woods.activity, Activity::Scavenge, "Scavenge");
        let maps = self.memories.colony().maps.clone();
        ui.add_enabled_ui(!maps.is_empty(), |ui| {
            ui.selectable_value(&mut self.woods.activity, Activity::Treasure, "Follow a map")
                .on_hover_text(match maps.len() {
                    0 => "A torn map turns up now and then on the old track.".to_owned(),
                    count => format!("{count} torn map{} to follow", plural(count)),
                });
        });
        if self.woods.activity == Activity::Treasure && maps.len() > 1 {
            let cast = &self.arrival.cast;
            let label = |index: usize| {
                let by = maps[index]
                    .found_by
                    .parse::<u64>()
                    .ok()
                    .and_then(|id| cast.member(id))
                    .map_or("a companion", |member| member.name.as_str());
                format!("Map {} (found by {by})", index + 1)
            };
            egui::ComboBox::from_id_salt("map")
                .selected_text(label(self.woods.map.min(maps.len() - 1)))
                .show_ui(ui, |ui| {
                    for index in 0..maps.len() {
                        ui.selectable_value(&mut self.woods.map, index, label(index));
                    }
                });
        }
    }
}

/// Did what the chosen hand does to whatever was clicked in a heap: or put back, picked up, or
/// gone to another heap if `may_go`.
fn heap_click(
    ground: &mut Playground,
    scavenge: &mut Scavenge,
    pointer: Option<(f32, f32)>,
    now: f32,
    may_go: bool,
) {
    let Some((x, y)) = pointer else {
        return;
    };
    if let Some(slot) = scavenge.basket_slot_at(x, y, super::SCENE_WIDTH) {
        scavenge.put_back(slot);
        return;
    }
    if let Some((heap, hidden)) = scavenge.open_at(x, y)
        && scavenge.at() == Some(heap)
    {
        scavenge.take(ground, heap, hidden, now);
        return;
    }
    if let Some((heap, item)) = scavenge.item_at(x, y)
        && scavenge.at() == Some(heap)
    {
        scavenge.act(ground, item, now);
        return;
    }
    if may_go && let Some(heap) = scavenge.heap_at(x, y) {
        scavenge.choose(ground, heap, now);
    }
}

/// What is under the pointer in a heap, and where to say so.
pub(super) fn heap_hover(
    scavenge: &Scavenge,
    pointer: Option<(f32, f32)>,
    may_go: bool,
) -> Option<(String, (f32, f32))> {
    if matches!(scavenge.phase(), Phase::Leaving { .. } | Phase::Over) {
        return None;
    }
    let (x, y) = pointer?;
    if let Some(slot) = scavenge.basket_slot_at(x, y, super::SCENE_WIDTH) {
        let find = scavenge.basket()[slot];
        let name = finds::find(find).map_or("it", |find| find.name);
        return Some((format!("Put back {}", lower(name)), (x, y + 14.0)));
    }
    if let Some((heap, hidden)) = scavenge.open_at(x, y) {
        let what = match scavenge.heap(heap)?.hidden[hidden].what {
            crate::track::heap::Hoard::Find(find) => find.name,
            crate::track::heap::Hoard::Map => "A torn map",
        };
        return Some((format!("Pick up {}", lower(what)), (x, y - 10.0)));
    }
    if let Some((heap, item)) = scavenge.item_at(x, y) {
        let site = scavenge.site(heap)?;
        let thing = &scavenge.heap(heap)?.items[item];
        let (left, top, w, _) = crate::track::stuff::scene_rect(site, thing);
        let label = if scavenge.at() == Some(heap) {
            scavenge.describe(heap, item)
        } else {
            site.name.to_owned()
        };
        return Some((label, ((left + w / 2) as f32, (top - 8) as f32)));
    }
    if !may_go {
        return None;
    }
    let heap = scavenge.heap_at(x, y)?;
    let site = scavenge.site(heap)?;
    (scavenge.at() != Some(heap)).then(|| {
        (
            site.name.to_owned(),
            (
                (site.left + site.width / 2) as f32,
                (site.ground - 40) as f32,
            ),
        )
    })
}

/// The light left, and how full the basket is (and the chest, if anything came out of one).
fn light_and_basket(ui: &mut egui::Ui, light: f32, basket: usize, chest: usize) {
    ui.add(
        egui::ProgressBar::new(light)
            .desired_width(70.0)
            .text("light"),
    );
    ui.label(format!("Basket {basket}/{BASKET}"));
    if chest > 0 {
        ui.label("The chest!");
    }
}

/// Who can do what at a heap: one to choose for each thing the party can do.
pub(super) fn hands_bar(ui: &mut egui::Ui, cast: &Cast, scavenge: &mut Scavenge) {
    let hands = scavenge.hands();
    let mut chosen = scavenge.hand();
    for (index, hand) in hands.iter().enumerate() {
        let (label, about) = hand_label(cast, scavenge, *hand);
        if ui
            .selectable_label(chosen == index, label)
            .on_hover_text(about)
            .clicked()
        {
            chosen = index;
        }
    }
    scavenge.set_hand(chosen);
}

/// What to call one thing the party can do, and what it means.
fn hand_label(cast: &Cast, scavenge: &Scavenge, hand: Hand) -> (String, &'static str) {
    let name = cast
        .member(hand.who())
        .map_or("Someone", |member| member.name.as_str());
    let character = scavenge.crew().character(hand.who());
    match hand {
        Hand::Lift(_) => {
            let strong = character.is_some_and(crew::strong);
            match character.map(crew::how) {
                Some(How::Gentle) => (
                    format!("{name} eases out"),
                    "Eases things out from under others without bringing the heap down: slower, \
                     and nothing cracks.",
                ),
                Some(How::Yank) => (
                    format!("{name} yanks"),
                    "Yanks things out: quick and cheap, but anything breakable inside gets \
                     jolted.",
                ),
                _ if strong => (
                    format!("{name} lifts"),
                    "Lifts things off, beams and boulders too. Pulled from under others, \
                     the heap comes down.",
                ),
                _ => (
                    format!("{name} lifts"),
                    "Lifts things off. Pulled from under others, the heap comes down.",
                ),
            }
        }
        Hand::Peek(_) => (
            format!("{name} peeks"),
            "Peeks under something and into it, without moving anything.",
        ),
        Hand::Squeeze(_) => (
            format!("{name} squeezes in"),
            "Squeezes into the gap under a board and brings out what is there, whole.",
        ),
    }
}

/// What to tell the person about something that happened at a heap.
pub(super) fn heap_line(cast: &Cast, event: Event) -> Option<String> {
    let name = |id: Id| cast.member(id).map_or("Someone", |m| m.name.as_str());
    Some(match event {
        Event::Lifted { who, stuff, how } => match how {
            How::Gentle => format!("{} eased {} out.", name(who), the(stuff)),
            How::Yank => format!("{} yanked {} out!", name(who), the(stuff)),
            How::Plain => format!("{} lifted {} off.", name(who), the(stuff)),
        },
        Event::Together { who, with, stuff } => format!(
            "{} and {} heaved {} off together.",
            name(who),
            name(with),
            the(stuff)
        ),
        Event::TooHeavy { who, stuff } => {
            format!(
                "{} is too heavy for {} alone.",
                capital(&the(stuff)),
                name(who)
            )
        }
        Event::Tumbled { count } => format!(
            "The heap shifted: {} came tumbling down.",
            if count == 1 {
                "something".to_owned()
            } else {
                format!("{count} things")
            }
        ),
        Event::Cracked => "Something cracked as the heap came down on it.".to_owned(),
        Event::Got { find } => format!("Found! {}.", find_name(find)),
        Event::Map { who } => format!(
            "{} found a torn map! It can be followed another day.",
            name(who)
        ),
        Event::Peeked {
            who,
            stuff,
            first,
            count,
        } => match (first, count) {
            (None, _) | (_, 0) => format!("{} peeked: nothing under {}.", name(who), the(stuff)),
            (Some(first), 1) => format!(
                "{} peeked under {}: {}.",
                name(who),
                the(stuff),
                lower(first)
            ),
            (Some(first), _) => format!(
                "{} peeked under {}: {}, and more.",
                name(who),
                the(stuff),
                lower(first)
            ),
        },
        Event::Squeezed { who, count: 0 } => {
            format!("{} squeezed in under there: nothing.", name(who))
        }
        Event::Squeezed { who, .. } => {
            format!("{} squeezed in and brought it out whole!", name(who))
        }
        Event::NoGap { who } => format!("No gap under that for {} to squeeze into.", name(who)),
        Event::Looked { who, find } => format!(
            "Found {}! {} looks it over very carefully.",
            lower(find_name(find)),
            name(who)
        ),
        Event::Nothing { stuff } => format!("Nothing under {}.", the(stuff)),
        Event::BasketFull => {
            "The basket's full. Put something back to make room, or head home.".to_owned()
        }
        Event::PutBack { find } => {
            format!("Put back: {}. Left on the track.", lower(find_name(find)))
        }
        Event::Dusk => {
            "The light is going. Deep in the heaps, something rare glints\u{2026}".to_owned()
        }
        Event::Leaving(Ending::Dusk) => "The light's gone. Time to head home.".to_owned(),
        Event::Leaving(Ending::Chose) => "Heading home.".to_owned(),
    })
}

/// "the plank", from "a plank".
fn the(stuff: Stuff) -> String {
    let name = stuff.name();
    let bare = name
        .strip_prefix("a ")
        .or_else(|| name.strip_prefix("an "))
        .unwrap_or(name);
    format!("the {bare}")
}

fn find_name(id: &str) -> &'static str {
    finds::find(id).map_or("something", |find| find.name)
}

/// A find's name as it reads mid-sentence.
fn lower(name: &str) -> String {
    let mut name = name.to_owned();
    if let Some(first) = name.get_mut(0..1) {
        first.make_ascii_lowercase();
    }
    name
}

fn capital(text: &str) -> String {
    let mut letters = text.chars();
    letters
        .next()
        .map(|first| first.to_uppercase().chain(letters).collect())
        .unwrap_or_default()
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// What sort of scavenger a companion makes, and whether it has been yet, for the person choosing
/// who to bring.
pub(super) fn scavenger_hint(colony: &ColonyMemories, id: Id, character: &Character) -> String {
    let been = match colony
        .scavenges_by
        .get(&id.to_string())
        .copied()
        .unwrap_or(0)
    {
        0 => "Never been scavenging yet.".to_owned(),
        1 => "Been scavenging once.".to_owned(),
        count => format!("{count} scavenges so far."),
    };
    format!("{} {been}", crew::scavenger(character))
}

/// How a companion reads a map, for the person choosing who to bring on a treasure hunt.
pub(super) fn reader_hint(character: &Character) -> String {
    crew::reader(character)
}

/// The old track's part of the journal: what has been found there, by whom first, and hints of
/// the rest: in a heap, deep in one, or only in a chest at the end of a map. And any maps held.
pub(super) fn journal(ui: &mut egui::Ui, colony: &ColonyMemories, who: &dyn Fn(&str) -> String) {
    let found = SCAVENGED
        .iter()
        .filter(|find| colony.finds.contains_key(find.id))
        .count();
    ui.add_space(6.0);
    ui.strong(format!(
        "Found on the old track ({found} of {}) \u{b7} {} scavenge{} \u{b7} {} chest{} dug up",
        SCAVENGED.len(),
        colony.scavenges,
        plural(colony.scavenges as usize),
        colony.chests,
        plural(colony.chests as usize)
    ));
    if !colony.maps.is_empty() {
        ui.label(
            egui::RichText::new(format!(
                "{} torn map{} to follow, from the Woods bar.",
                colony.maps.len(),
                plural(colony.maps.len())
            ))
            .italics(),
        );
    }
    for find in &SCAVENGED {
        match colony.finds.get(find.id) {
            Some(record) => {
                ui.label(egui::RichText::new(find.name).strong());
                ui.label(egui::RichText::new(find.blurb).small());
                ui.label(
                    egui::RichText::new(format!(
                        "Becomes {} \u{b7} brought home {}\u{d7} \u{b7} first found with {}",
                        find.piece.to_lowercase(),
                        record.count,
                        who(&record.first_by)
                    ))
                    .small()
                    .italics(),
                );
            }
            None => {
                let whereabouts = match find.tier {
                    Tier::Exceptional => "in a chest, at the end of a map",
                    Tier::Rare => "deep in a heap, or in a chest",
                    _ => "in a heap on the old track",
                };
                ui.label(
                    egui::RichText::new(format!(
                        "??? \u{b7} something {} {whereabouts}",
                        find.tier.label()
                    ))
                    .weak(),
                );
            }
        }
    }
}
