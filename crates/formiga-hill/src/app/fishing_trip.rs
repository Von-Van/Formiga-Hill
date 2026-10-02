//! Fishing at the pool, in the window: setting off, casting and striking, reeling in, and
//! remembering what was caught.

use super::HillApp;
use crate::cast::Id;
use crate::finds;
use crate::fishing::angling::{Angling, Event, Outset, Phase};
use crate::fishing::{self, fish};
use eframe::egui;
use formiga_art::Canvas;

impl HillApp {
    /// Sets off for the pool with whoever was chosen; the first fishes.
    pub(super) fn set_off_fishing(&mut self, now: f32) {
        let party = self.woods.party.clone();
        if party.is_empty() {
            return;
        }
        let cast = &self.arrival.cast;
        let colony = self.memories.colony();
        let seed = cast.snapshot.colony_id.bytes().fold(
            u64::from(colony.fishing_trips) << 32 | 0xf15,
            |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3),
        );
        let mut ground = fishing::open(cast, &party, now);
        let outset = Outset {
            party,
            drought: colony.fish_drought,
            seed,
        };
        let caught = |id: &str| colony.fish.contains_key(id);
        let angling = Angling::new(&mut ground, outset, caught, now);
        self.woods.fishing = Some((ground, angling));
        self.notice = Some((
            "Each kind of fish keeps to its own part of the pool. Click the water to cast."
                .to_owned(),
            now,
        ));
    }

    /// Plays the trip on; `holding` reels in while something is on the line.
    pub(super) fn tick_fishing(&mut self, now: f32, holding: bool) {
        let cast = &self.arrival.cast;
        let Some((ground, angling)) = &mut self.woods.fishing else {
            return;
        };
        ground.tick(cast, now);
        angling.hold(holding);
        angling.tick(ground, now);
        let name = |id: Id| {
            cast.member(id)
                .map_or("Someone", |member| member.name.as_str())
        };
        let kind = |id: &str| fish::fish(id).map_or("a fish", |fish| fish.name);
        for event in angling.take_events() {
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
            self.notice = Some((line, now));
        }
        if angling.phase() == Phase::Over {
            self.finish_fishing(now);
        }
    }

    /// Brings the day's catch home to the journal: fish let go, snags to the satchel.
    pub(super) fn finish_fishing(&mut self, now: f32) {
        let Some((_, angling)) = self.woods.fishing.take() else {
            return;
        };
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
    }

    pub(super) fn compose_pool(&mut self, now: f32) -> Option<Canvas> {
        let (ground, angling) = self.woods.fishing.as_mut()?;
        let mut scene = ground.compose(now);
        angling.draw(&mut scene, now);
        Some(scene)
    }

    /// A click at the pool: cast where it points, or strike.
    pub(super) fn fishing_click(&mut self, pointer: Option<(f32, f32)>, now: f32) {
        let Some((ground, angling)) = &mut self.woods.fishing else {
            return;
        };
        match angling.phase() {
            Phase::Ready => {
                if let Some(at) = pointer {
                    angling.cast(ground, at, now);
                }
            }
            Phase::Waiting { .. } => angling.strike(ground, now),
            _ => {}
        }
    }

    /// The key for striking, for anyone not using the pointer.
    pub(super) fn fishing_strike(&mut self, now: f32) {
        if let Some((ground, angling)) = &mut self.woods.fishing
            && matches!(angling.phase(), Phase::Waiting { .. })
        {
            angling.strike(ground, now);
        }
    }

    /// Which part of the pool the pointer is over, while choosing where to cast.
    pub(super) fn fishing_hover(
        &self,
        pointer: Option<(f32, f32)>,
    ) -> Option<(String, (f32, f32))> {
        let (_, angling) = self.woods.fishing.as_ref()?;
        if angling.phase() != Phase::Ready {
            return None;
        }
        let (x, y) = pointer?;
        let haunt = Angling::haunt_at(x, y)?;
        Some((haunt.name().to_owned(), (x, y - 10.0)))
    }

    pub(super) fn fishing_bar(&mut self, ui: &mut egui::Ui, now: f32) {
        let Some((_, angling)) = &self.woods.fishing else {
            return;
        };
        ui.add(
            egui::ProgressBar::new(angling.light_left())
                .desired_width(70.0)
                .text("light"),
        );
        ui.label(format!("Landed {}", angling.creel().len()));
        let hint = match angling.phase() {
            Phase::Ready => "Click the water to cast.",
            Phase::Waiting { .. } => "Strike (click or Space) when the float goes under.",
            Phase::Fighting(fight) if fight.strain > 0.7 => "Ease off!",
            Phase::Fighting(_) => "Hold to reel in.",
            _ => "",
        };
        if !hint.is_empty() {
            ui.label(egui::RichText::new(hint).strong());
        }
        let leaving = matches!(angling.phase(), Phase::Leaving { .. } | Phase::Over);
        if ui
            .add_enabled(!leaving, egui::Button::new("Head home"))
            .clicked()
            && let Some((ground, angling)) = &mut self.woods.fishing
        {
            angling.head_home(ground, now);
        }
    }
}
