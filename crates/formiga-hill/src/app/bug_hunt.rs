//! Bug catching in the meadow, in the window: setting off, choosing a bug and creeping up on it,
//! swinging the net, and remembering what was caught.

use super::outing::{Outing, Told};
use super::{HillApp, paper};
use crate::audio::Cue;
use crate::cast::{Cast, Id};
use crate::finds;
use crate::meadow::catching::{Event, Hunt, Outset, Phase, Why};
use crate::meadow::{self, bugs};
use crate::playground::Playground;
use eframe::egui;

impl HillApp {
    /// Sets off for the meadow with whoever was chosen; the first carries the net.
    pub(super) fn set_off_bug_hunting(&mut self, now: f32) {
        let party = self.woods.party.clone();
        if party.is_empty() {
            return;
        }
        let cast = &self.arrival.cast;
        let colony = self.memories.colony();
        let close_pair = cast.close_pair(&party);
        let seed = self.outing_seed(colony.bug_hunts, 0xb06);
        let mut ground = meadow::open(cast, &party, now, &colony.hilltop);
        ground.set_daylight(self.daylight);
        let influence = crate::woods::influence(&colony.hilltop);
        let note = influence.notes.first().copied();
        let outset = Outset {
            party,
            drought: colony.bug_drought,
            close_pair,
            influence,
            seed,
        };
        let caught = |id: &str| colony.bugs.contains_key(id);
        let found = |id: &str| colony.finds.contains_key(id);
        let mut hunt = Hunt::new(&mut ground, outset, caught, found, now);
        hunt.set_hour_dark(self.daylight.darkness());
        self.woods.outing = Some(Outing::BugHunt(ground, hunt));
        let line = note.unwrap_or(
            "Click a bug to go after it. Hold to creep closer, let go to keep still, and Space \
             to swing the net.",
        );
        self.notice = Some((line.to_owned(), now));
    }

    /// Brings the day's catch home to the journal: bugs let go, anything else to the satchel.
    pub(super) fn finish_bug_hunt(&mut self, hunt: &Hunt, now: f32) {
        let haul = self
            .memories
            .back_from_the_meadow(hunt.party(), hunt.jar(), hunt.basket());
        let caught = hunt.jar().len();
        let mut line = match caught {
            0 => "Home from the meadow. Nothing caught this time, but there was a lot of creeping."
                .to_owned(),
            1 => "Home from the meadow with one bug caught and let go.".to_owned(),
            count => format!("Home from the meadow with {count} bugs caught and let go."),
        };
        if !haul.new_kinds.is_empty() {
            line.push_str(&format!(" New to the journal: {}.", haul.new_kinds.len()));
        }
        if !haul.longest_yet.is_empty() {
            line.push_str(" A biggest-yet!");
        }
        if !hunt.basket().is_empty() {
            line.push_str(" Something from the net is in the satchel, for the Hilltop.");
        }
        self.notice = Some((line, now));
        self.grant(super::departures::Outing::BugCatching, hunt.party(), now);
    }
}

/// What the hunt's events sound like, and say.
pub(super) fn told(cast: &Cast, events: Vec<Event>) -> Vec<Told> {
    let name = |id: Id| {
        cast.member(id)
            .map_or("Someone", |member| member.name.as_str())
    };
    let kind = |id: &str| bugs::bug(id).map_or("a bug", |bug| bug.name);
    let lower = |id: &str| super::lower(kind(id));
    events
        .into_iter()
        .map(|event| {
            let cue = super::sound::hunted(&event);
            let line = match event {
                Event::Stalking { bug } => {
                    format!(
                        "After {}. Hold to creep closer; let go to keep still.",
                        lower(bug)
                    )
                }
                Event::Startled { bug } => format!("{} saw you coming, and was off!", kind(bug)),
                Event::Missed { bug, why } => match why {
                    Why::InFlight => format!("Missed: {} was on the move.", lower(bug)),
                    Why::WingsOpen => {
                        format!(
                            "{} saw the net coming. Wings open, it was watching.",
                            kind(bug)
                        )
                    }
                    Why::Chirping => {
                        format!("Too late: {} had started to chirp, and jumped.", lower(bug))
                    }
                    Why::Dark => format!("{} went dark just as the net came down.", kind(bug)),
                },
                Event::OutOfReach => "Not close enough yet.".to_owned(),
                Event::Caught { bug, size } => {
                    format!(
                        "Caught! {}, {size:.0} mm across. Admired, and let go.",
                        kind(bug)
                    )
                }
                Event::Netted { find } => {
                    let find = finds::find(find).map_or("something", |find| find.name);
                    format!("Not the bug, but something else in the net: {find}!")
                }
                Event::LandedOn { who, bug } => {
                    format!(
                        "{} has landed on {}, who is keeping very still.",
                        kind(bug),
                        name(who)
                    )
                }
                Event::Pointed { who, bug } => {
                    format!("{} has spotted {}!", name(who), lower(bug))
                }
                Event::Dusk => {
                    "The light is going. Something pale is stirring in the brambles\u{2026}"
                        .to_owned()
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

/// A click in the meadow: choose the bug under it, or swing at the chosen one in reach.
pub(super) fn click(
    ground: &mut Playground,
    hunt: &mut Hunt,
    pointer: Option<(f32, f32)>,
    now: f32,
) -> Option<Cue> {
    let (x, y) = pointer?;
    let chosen = hunt.target().map(|(_, at)| at);
    let on_chosen = chosen.is_some_and(|at| crate::playground::distance(at, (x, y)) <= 9.0);
    if on_chosen && hunt.in_reach(ground) {
        hunt.swing(ground, now);
        Some(Cue::Swish)
    } else {
        hunt.choose(ground, x, y, now);
        None
    }
}

/// The bug under the pointer, named.
pub(super) fn hover(hunt: &Hunt, pointer: Option<(f32, f32)>) -> Option<(String, (f32, f32))> {
    if hunt.phase() != Phase::Hunting {
        return None;
    }
    let (x, y) = pointer?;
    let (name, at) = hunt.name_at(x, y)?;
    Some((name.to_owned(), (at.0, at.1 - 10.0)))
}

pub(super) fn tray(ui: &mut egui::Ui, ground: &mut Playground, hunt: &mut Hunt, now: f32) {
    paper::meter(ui, hunt.light_left(), "light");
    paper::slip(ui, format!("Caught {}", hunt.jar().len()));
    let hint = match (hunt.phase(), hunt.target()) {
        (Phase::Hunting, None) => "Click a bug to go after it.",
        (Phase::Hunting, Some(_)) if hunt.in_reach(ground) => "In reach! Space to swing.",
        (Phase::Hunting, Some(_)) => "Hold to creep closer.",
        _ => "",
    };
    if !hint.is_empty() {
        paper::hint(ui, hint);
    }
    let leaving = matches!(hunt.phase(), Phase::Leaving { .. } | Phase::Over);
    if ui
        .add(paper::Button::new("Head home").enabled(!leaving))
        .clicked()
    {
        hunt.head_home(ground, now);
    }
}
