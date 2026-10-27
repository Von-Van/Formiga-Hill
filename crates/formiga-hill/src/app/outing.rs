//! Whatever the party is out doing in the Woods. Only one outing is ever under way, so every
//! kind is one thing here: the ground it is played on and the game played there, with the steps
//! every outing shares (playing on, drawing, the pointer, the key, the tray, coming home) asked
//! of it in one place. A new kind of outing is a new kind below, and the compiler names each
//! step it still needs.

use super::{HillApp, bug_hunt, fishing_trip, rummaging};
use crate::audio::Cue;
use crate::cast::{Cast, Id};
use crate::expedition::legs::Input;
use crate::fishing::angling::{self, Angling};
use crate::meadow::catching;
use crate::playground::Playground;
use crate::woods::rummage::{self, Rummage};
use eframe::egui;
use formiga_art::Canvas;

/// An outing under way in the Woods, on its own ground.
pub enum Outing {
    /// Rummaging in the glade.
    Rummage(Playground, Rummage),
    /// Fishing at the pool; the first of the party fishes.
    Fishing(Playground, Angling),
    /// Catching bugs in the meadow; the first of the party carries the net.
    BugHunt(Playground, catching::Hunt),
}

/// Something that happened on an outing, as the window takes it: a sound to play, and a line
/// for the notice.
pub struct Told {
    pub cue: Option<Cue>,
    pub line: Option<String>,
}

impl Outing {
    /// Where the outing is.
    pub fn ground_mut(&mut self) -> &mut Playground {
        match self {
            Self::Rummage(ground, _) | Self::Fishing(ground, _) | Self::BugHunt(ground, _) => {
                ground
            }
        }
    }

    /// Who went, the first leading.
    pub fn party(&self) -> &[Id] {
        match self {
            Self::Rummage(_, rummage) => rummage.party(),
            Self::Fishing(_, angling) => angling.party(),
            Self::BugHunt(_, hunt) => hunt.party(),
        }
    }

    /// How dark the hour is, for the outing's own dusk.
    pub fn set_hour_dark(&mut self, dark: f32) {
        match self {
            Self::Rummage(_, rummage) => rummage.set_hour_dark(dark),
            Self::Fishing(_, angling) => angling.set_hour_dark(dark),
            Self::BugHunt(_, hunt) => hunt.set_hour_dark(dark),
        }
    }

    /// Plays the outing on, with whatever the person is holding down, and says what happened.
    pub fn tick(&mut self, cast: &Cast, input: Input, now: f32) -> Vec<Told> {
        match self {
            Self::Rummage(ground, rummage) => {
                ground.tick(cast, now);
                rummage.hold(ground, input.holding, input.dt, now);
                rummage.tick(ground, now);
                let events = rummage.take_events();
                rummaging::told(cast, rummage, events)
            }
            Self::Fishing(ground, angling) => {
                ground.tick(cast, now);
                angling.hold(input.holding);
                angling.tick(ground, now);
                let events = angling.take_events();
                fishing_trip::told(cast, angling, events)
            }
            Self::BugHunt(ground, hunt) => {
                ground.tick(cast, now);
                hunt.hold(input.creeping);
                hunt.tick(ground, now);
                let events = hunt.take_events();
                bug_hunt::told(cast, events)
            }
        }
    }

    /// Whether the outing is over, and the party home.
    pub fn over(&self) -> bool {
        match self {
            Self::Rummage(_, rummage) => rummage.phase() == rummage::Phase::Over,
            Self::Fishing(_, angling) => angling.phase() == angling::Phase::Over,
            Self::BugHunt(_, hunt) => hunt.phase() == catching::Phase::Over,
        }
    }

    /// Whether the party has followed the glint off the map.
    pub fn beckoned(&self) -> bool {
        matches!(self, Self::Rummage(_, rummage) if rummage.phase() == rummage::Phase::Beckoned)
    }

    /// The outing's scene as it is now; `pointer` marks whatever is under it.
    pub fn compose(&mut self, now: f32, pointer: Option<(f32, f32)>) -> Canvas {
        let _ = pointer;
        match self {
            Self::Rummage(ground, rummage) => {
                let mut scene = ground.compose(now);
                rummage.draw(&mut scene, now);
                scene
            }
            Self::Fishing(ground, angling) => {
                let mut scene = ground.compose(now);
                angling.draw(&mut scene, now);
                scene
            }
            Self::BugHunt(ground, hunt) => {
                ground.set_fliers(hunt.fliers(now));
                let mut scene = ground.compose(now);
                hunt.draw(&mut scene, ground, now);
                scene
            }
        }
    }

    /// A click in the outing's scene, as its game takes it. Says what to hear, if anything.
    pub fn click(&mut self, pointer: Option<(f32, f32)>, now: f32) -> Option<Cue> {
        match self {
            Self::Rummage(ground, rummage) => rummaging::click(ground, rummage, pointer, now),
            Self::Fishing(ground, angling) => fishing_trip::click(ground, angling, pointer, now),
            Self::BugHunt(ground, hunt) => bug_hunt::click(ground, hunt, pointer, now),
        }
    }

    /// The key for catching, striking or swinging, for anyone not using the pointer. Says what
    /// to hear, if anything.
    pub fn strike(&mut self, now: f32) -> Option<Cue> {
        match self {
            Self::Rummage(ground, rummage) => {
                if matches!(rummage.phase(), rummage::Phase::Catching(_)) && !ground.reduce_motion()
                {
                    rummage.strike(ground, now);
                }
                None
            }
            Self::Fishing(ground, angling) => {
                if matches!(angling.phase(), angling::Phase::Waiting { .. }) {
                    angling.strike(ground, now);
                }
                None
            }
            Self::BugHunt(ground, hunt) => {
                hunt.swing(ground, now);
                Some(Cue::Swish)
            }
        }
    }

    /// What to call whatever the pointer is over, and where to say so.
    pub fn hover(&self, pointer: Option<(f32, f32)>) -> Option<(String, (f32, f32))> {
        match self {
            Self::Rummage(_, rummage) => rummaging::hover(rummage, pointer),
            Self::Fishing(_, angling) => fishing_trip::hover(angling, pointer),
            Self::BugHunt(_, hunt) => bug_hunt::hover(hunt, pointer),
        }
    }

    /// The pointer over the outing's scene: where the party looks, and who is under it. At the
    /// pool and in the meadow the pointer only casts and chooses a bug, and nobody turns to it.
    pub fn point(&mut self, pointer: Option<(f32, f32)>, now: f32) -> Option<Id> {
        match self {
            Self::Rummage(ground, _) => {
                ground.set_pointer(pointer);
                pointer.and_then(|(x, y)| ground.actor_at(x, y, now))
            }
            Self::Fishing(..) | Self::BugHunt(..) => None,
        }
    }

    /// The outing's tray: the light left, what has been got so far, what to do, and the way
    /// home.
    pub fn tray(&mut self, ui: &mut egui::Ui, cast: &Cast, now: f32) {
        let _ = cast;
        match self {
            Self::Rummage(ground, rummage) => rummaging::tray(ui, ground, rummage, now),
            Self::Fishing(ground, angling) => fishing_trip::tray(ui, ground, angling, now),
            Self::BugHunt(ground, hunt) => bug_hunt::tray(ui, ground, hunt, now),
        }
    }
}

impl HillApp {
    /// Brings whatever the outing found home, into the journal and the satchel, if one is under
    /// way.
    pub(super) fn finish_outing(&mut self, now: f32) {
        let Some(outing) = self.woods.outing.take() else {
            return;
        };
        match outing {
            Outing::Rummage(_, rummage) => self.finish_rummage(&rummage, now),
            Outing::Fishing(_, angling) => self.finish_fishing(&angling, now),
            Outing::BugHunt(_, hunt) => self.finish_bug_hunt(&hunt, now),
        }
    }
}
