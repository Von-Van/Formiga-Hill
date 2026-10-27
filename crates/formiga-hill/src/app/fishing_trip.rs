//! Fishing at the pool, in the window: setting off, casting and striking, reeling in, and
//! remembering what was caught.

use super::outing::{Outing, Told};
use super::{HillApp, paper};
use crate::audio::Cue;
use crate::cast::{Cast, Id};
use crate::finds;
use crate::fishing::angling::{Angling, Event, Outset, Phase};
use crate::fishing::{self, fish};
use crate::playground::Playground;
use eframe::egui;

impl HillApp {
    /// Sets off for the pool with whoever was chosen; the first fishes.
    pub(super) fn set_off_fishing(&mut self, now: f32) {
        let party = self.woods.party.clone();
        if party.is_empty() {
            return;
        }
        let cast = &self.arrival.cast;
        let colony = self.memories.colony();
        let seed = self.outing_seed(colony.fishing_trips, 0xf15);
        let mut ground = fishing::open(cast, &party, now);
        let influence = crate::woods::influence(&colony.hilltop);
        let note = influence.notes.first().copied();
        let outset = Outset {
            party,
            drought: colony.fish_drought,
            influence,
            seed,
        };
        let caught = |id: &str| colony.fish.contains_key(id);
        let found = |id: &str| colony.finds.contains_key(id);
        let angling = Angling::new(&mut ground, outset, caught, found, now);
        self.woods.outing = Some(Outing::Fishing(ground, angling));
        let line = note.unwrap_or(
            "Each kind of fish keeps to its own part of the pool. Click the water to cast.",
        );
        self.notice = Some((line.to_owned(), now));
    }

    /// Brings the day's catch home to the journal: fish let go, snags to the satchel.
    pub(super) fn finish_fishing(&mut self, angling: &Angling, now: f32) {
        let haul =
            self.memories
                .back_from_fishing(angling.party(), angling.creel(), angling.basket());
        let landed = angling.creel().len();
        let mut line = match landed {
            0 => "Home from the pool. Nothing landed this time, but it was a lovely evening."
                .to_owned(),
            1 => "Home from the pool with one fish landed and let go.".to_owned(),
            count => format!("Home from the pool with {count} fish landed and let go."),
        };
        if !haul.new_kinds.is_empty() {
            line.push_str(&format!(" New to the journal: {}.", haul.new_kinds.len()));
        }
        if !haul.longest_yet.is_empty() {
            line.push_str(" A longest-yet!");
        }
        if !angling.basket().is_empty() {
            line.push_str(" A snag is in the satchel, for the Hilltop.");
        }
        self.notice = Some((line, now));
        self.grant(super::departures::Outing::Fishing, angling.party(), now);
    }
}

/// What the trip's events sound like, and say.
pub(super) fn told(cast: &Cast, angling: &Angling, events: Vec<Event>) -> Vec<Told> {
    let name = |id: Id| {
        cast.member(id)
            .map_or("Someone", |member| member.name.as_str())
    };
    let kind = |id: &str| fish::fish(id).map_or("a fish", |fish| fish.name);
    events
        .into_iter()
        .map(|event| {
            let cue = super::sound::angled(&event);
            let line = match event {
                Event::Spooked { fish } => format!("{} darted off at the splash.", kind(fish)),
                Event::Nibble => "A nibble\u{2026}".to_owned(),
                Event::Bite => "It's under! Strike!".to_owned(),
                Event::TooSoon => "Too soon! That was only a nibble.".to_owned(),
                Event::Stolen => "Too late: it took the bait and went.".to_owned(),
                Event::Hooked { .. } => {
                    "Hooked! Hold to reel in, and ease off when it pulls.".to_owned()
                }
                Event::Snapped => "Snap! It got away.".to_owned(),
                Event::SlippedOff => "In the dusk, it slipped quietly off the hook.".to_owned(),
                Event::Landed { fish, length } => {
                    format!("{}, {length:.1} cm! Admired, and let go.", kind(fish))
                }
                Event::Snagged => {
                    let find = angling
                        .basket()
                        .last()
                        .and_then(|id| finds::find(id))
                        .map_or("something", |find| find.name);
                    format!("Something snagged on the line: {find}!")
                }
                Event::Pointed { who, haunt } => {
                    format!("{} spotted a big one by {}.", name(who), haunt.name())
                }
                Event::Dusk => {
                    "The light is going. Something stirs in the deep water\u{2026}".to_owned()
                }
                Event::Leaving => "Time to head home.".to_owned(),
            };
            Told {
                cue,
                line: Some(line),
            }
        })
        .collect()
}

/// A click at the pool: cast where it points, or strike.
pub(super) fn click(
    ground: &mut Playground,
    angling: &mut Angling,
    pointer: Option<(f32, f32)>,
    now: f32,
) -> Option<Cue> {
    match angling.phase() {
        Phase::Ready => {
            let at = pointer?;
            angling.cast(ground, at, now);
            Some(Cue::Cast)
        }
        Phase::Waiting { .. } => {
            angling.strike(ground, now);
            None
        }
        _ => None,
    }
}

/// Which part of the pool the pointer is over, while choosing where to cast.
pub(super) fn hover(
    angling: &Angling,
    pointer: Option<(f32, f32)>,
) -> Option<(String, (f32, f32))> {
    if angling.phase() != Phase::Ready {
        return None;
    }
    let (x, y) = pointer?;
    let haunt = Angling::haunt_at(x, y)?;
    Some((haunt.name().to_owned(), (x, y - 10.0)))
}

pub(super) fn tray(ui: &mut egui::Ui, ground: &mut Playground, angling: &mut Angling, now: f32) {
    paper::meter(ui, angling.light_left(), "light");
    paper::slip(ui, format!("Landed {}", angling.creel().len()));
    let hint = match angling.phase() {
        Phase::Ready => "Click the water to cast.",
        Phase::Waiting { .. } => "Strike (click or Space) when the float goes under.",
        Phase::Fighting(fight) if fight.strain > 0.7 => "Ease off!",
        Phase::Fighting(_) => "Hold to reel in.",
        _ => "",
    };
    if !hint.is_empty() {
        paper::hint(ui, hint);
    }
    let leaving = matches!(angling.phase(), Phase::Leaving { .. } | Phase::Over);
    if ui
        .add(paper::Button::new("Head home").enabled(!leaving))
        .clicked()
    {
        angling.head_home(ground, now);
    }
}
