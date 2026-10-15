//! Woods adventures in the window: choosing one, setting off on it, what is told along the way,
//! and bringing it home. An adventure is walked as an expedition is, on its own map, so the map,
//! the stops and the basket are the expedition's own controls; only setting off, the author's
//! words and what is kept at the end are here.

use super::{HillApp, capital, listed, paper, plural};
use crate::adventure::{self, Adventure};
use crate::cast::{Cast, Id};
use crate::expedition::map::{Opener, Way};
use crate::expedition::{self, Event, Expedition, Outset};
use crate::memories::AdventureHome;
use crate::story::{Origin, Package};
use crate::woods;
use eframe::egui;

/// An adventure under way.
pub struct Playing {
    /// The adventure as `<package id>/<adventure id>`, as memories keep it.
    pub key: String,
    pub adventure: Adventure,
    /// Being tried out by whoever is making it: nothing that happens is kept.
    pub trying: bool,
}

impl HillApp {
    /// Every adventure the Woods offer: Hill's own, then any in packages not set aside.
    pub(super) fn adventures(&self) -> Vec<(&Package, &Adventure)> {
        self.library
            .adventures()
            .filter(|(package, _)| !self.set_aside.contains(&package.id))
            .collect()
    }

    /// The adventure chosen to go on next: the one picked, or else the first.
    pub(super) fn chosen_adventure(&self) -> Option<(&Package, &Adventure)> {
        let adventures = self.adventures();
        let picked = self.woods.adventure.as_ref().and_then(|(package, id)| {
            adventures
                .iter()
                .find(|(p, a)| &p.id == package && &a.id == id)
                .copied()
        });
        picked.or_else(|| adventures.first().copied())
    }

    /// Which adventure to go on, on the Woods' tray.
    pub(super) fn adventure_choice(&mut self, ui: &mut egui::Ui) {
        let finished = &self.memories.colony().adventures;
        let entries: Vec<(String, String, String, String)> = self
            .adventures()
            .into_iter()
            .map(|(package, adventure)| {
                let key = format!("{}/{}", package.id, adventure.id);
                let mut about = adventure.about.clone();
                if self.library.origin(&package.id) != Some(Origin::Official) {
                    about = format!("{about} By {}.", package.author).trim().to_owned();
                }
                if finished.get(&key).is_some_and(|record| record.finished > 0) {
                    about.push_str(" Finished before.");
                }
                (
                    package.id.clone(),
                    adventure.id.clone(),
                    adventure.title.clone(),
                    about,
                )
            })
            .collect();
        let Some((package, id)) = self
            .chosen_adventure()
            .map(|(package, adventure)| (package.id.clone(), adventure.id.clone()))
        else {
            paper::hint(ui, "No adventures yet: make one!");
            return;
        };
        let title = entries
            .iter()
            .find(|(p, a, ..)| *p == package && *a == id)
            .map_or_else(String::new, |(.., title, _)| title.clone());
        let (_, picked) = paper::menu(
            ui,
            paper::Button::new(format!("{title} \u{25b4}")),
            false,
            |ui| {
                let mut picked = None;
                for (p, a, title, about) in &entries {
                    let pick = ui.add(paper::Button::new(title).selected(*p == package && *a == id));
                    if paper::explain(ui, pick, about).clicked() {
                        picked = Some((p.clone(), a.clone()));
                    }
                }
                picked
            },
        );
        if let Some(Some(picked)) = picked {
            self.woods.adventure = Some(picked);
        }
    }

    /// Sets off on the chosen adventure with whoever was chosen, up to three.
    pub(super) fn set_off_adventure(&mut self, now: f32) {
        let Some((package, adventure)) = self.chosen_adventure() else {
            return;
        };
        let playing = Playing {
            key: format!("{}/{}", package.id, adventure.id),
            adventure: adventure.clone(),
            trying: false,
        };
        self.go_on(playing, now);
    }

    /// Tries out an adventure being made, with whoever was chosen: nothing is kept.
    pub(super) fn try_adventure(&mut self, adventure: Adventure, now: f32) {
        let playing = Playing {
            key: String::new(),
            adventure,
            trying: true,
        };
        self.go_on(playing, now);
    }

    fn go_on(&mut self, playing: Playing, now: f32) {
        let party: Vec<Id> = self
            .woods
            .party
            .iter()
            .copied()
            .take(expedition::PARTY)
            .collect();
        if party.is_empty() {
            return;
        }
        let known = self.known();
        let cast = &self.arrival.cast;
        let colony = self.memories.colony();
        let played = colony
            .adventures
            .get(&playing.key)
            .map_or(0, |record| record.played);
        // A different day each time it is played, the same for the same colony.
        let seed = cast
            .snapshot
            .colony_id
            .bytes()
            .chain(playing.key.bytes())
            .fold(u64::from(played) << 32 | 0xad7, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
            });
        let outset = Outset {
            map: playing.adventure.map.clone(),
            party,
            influence: woods::influence(&colony.hilltop),
            seed,
        };
        let mut expedition = Expedition::new(cast, outset, known, now);
        expedition.set_daylight(self.daylight);
        let title = playing.adventure.title.clone();
        let line = if playing.trying {
            format!("Trying out {title}: nothing found is kept. Click a place on the map to go there.")
        } else if playing.adventure.about.is_empty() {
            format!("{title}. Click a place on the map to go there, and where you are to stop.")
        } else {
            format!("{} Click a place on the map to go there.", playing.adventure.about)
        };
        self.woods.expedition = Some(expedition);
        self.woods.playing = Some(playing);
        self.notice = Some((line, now));
    }

    /// Brings an adventure home: whatever is in the basket into the journal and the satchel,
    /// as an expedition's is, and whether it was finished. A try keeps nothing.
    pub(super) fn finish_adventure(&mut self, expedition: Expedition, playing: Playing, now: f32) {
        let finished = expedition.finished();
        let title = playing.adventure.title.clone();
        if playing.trying {
            let line = if finished {
                format!("That's {title} tried, ending and all. Nothing was kept.")
            } else {
                format!("Back from trying {title}. Nothing was kept.")
            };
            self.notice = Some((line, now));
            return;
        }
        let found: Vec<(&str, u64)> = expedition
            .basket()
            .iter()
            .map(|carried| (carried.find, carried.by))
            .collect();
        let home = AdventureHome {
            adventure: &playing.key,
            finished,
            found: &found,
            fish: expedition.fish(),
            bugs: expedition.bugs(),
            maps: expedition.maps(),
        };
        let new = self.memories.back_from_an_adventure(&home);
        let mut line = if finished {
            format!("Home from {title}, the end reached!")
        } else {
            format!("Home from {title}, the end still to find another day.")
        };
        match (found.len(), new.len()) {
            (0, _) => {}
            (count, 0) => line.push_str(&format!(
                " {count} find{} in the satchel, for the Hilltop.",
                plural(count)
            )),
            (count, fresh) => line.push_str(&format!(
                " {count} find{} in the satchel, {fresh} new to the journal.",
                plural(count)
            )),
        }
        let caught = expedition.fish().len() + expedition.bugs().len();
        if caught > 0 {
            line.push_str(&format!(" {caught} caught and let go on the way."));
        }
        if !expedition.maps().is_empty() {
            line.push_str(" And a torn map, to follow another day.");
        }
        self.notice = Some((line, now));
    }
}

/// What to tell the person of something on an adventure: the author's words on arriving
/// somewhere and at an ending, and the ways only some company takes as this map names them;
/// anything else is told as on any expedition.
pub(super) fn told(
    expedition: &Expedition,
    playing: Option<&Playing>,
    cast: &Cast,
    event: Event,
) -> Option<String> {
    let Some(playing) = playing else {
        return super::expedition::tell(expedition, cast, event);
    };
    let adventure = &playing.adventure;
    let leader = expedition.party().first().copied();
    let (name, kind) = leader
        .and_then(|id| cast.member(id))
        .map_or(("Someone".to_owned(), None), |member| {
            (
                member.name.clone(),
                Some(crate::character::Character::of(member).kind),
            )
        });
    let name_of = |id: Id| {
        cast.member(id)
            .map_or("Someone".to_owned(), |member| member.name.clone())
    };
    match event {
        Event::Opened { opener, who } => {
            let ways: Vec<String> = adventure
                .map
                .paths
                .iter()
                .filter_map(|path| match &path.way {
                    Way::Shortcut(called, by) if *by == opener => Some(called.to_string()),
                    _ => None,
                })
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            if ways.is_empty() {
                return None;
            }
            let who = match (opener, expedition.company().close_pair) {
                (Opener::ClosePair, Some((a, b))) => format!("{} and {}", name_of(a), name_of(b)),
                _ => name_of(who),
            };
            Some(format!("{who} can take {}.", listed(&ways)))
        }
        Event::Arrived { place } => {
            let usual = super::expedition::tell(expedition, cast, event);
            match adventure.arrivals.get(place).and_then(Option::as_ref) {
                Some(line) => {
                    let said = adventure::told(line, &name, kind);
                    Some(match usual {
                        // Keep the hint of what can be done here.
                        Some(usual) => match usual.split_once(". ") {
                            Some((_, hint)) => format!("{said} {}", capital(hint)),
                            None => said,
                        },
                        None => said,
                    })
                }
                None => usual,
            }
        }
        Event::Finished { place } => {
            let said = adventure
                .endings
                .get(place)
                .and_then(Option::as_ref)
                .map(|line| adventure::told(line, &name, kind))
                .unwrap_or_default();
            Some(
                format!("{said} The end! Head home whenever you like.")
                    .trim()
                    .to_owned(),
            )
        }
        _ => super::expedition::tell(expedition, cast, event),
    }
}
