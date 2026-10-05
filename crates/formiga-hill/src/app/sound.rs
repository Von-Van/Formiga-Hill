//! Sound in the window: the piece for wherever the person is; what the train, the sack race, the
//! rope, the colony at work and a story are heard doing, listened for each frame; and the Sound
//! window, with its levels and mute. The cues for things that tell the window themselves are
//! played where they are told, beside their notices.

use super::{Area, HillApp};
use crate::audio::{Cue, Music, Track};
use crate::character::Offer;
use crate::clearing::sovereign::State;
use crate::station::Heard;
use eframe::egui;

// What the Woods are heard doing, from what each outing tells the window: a find, a splash, a
// flutter, a pick, treasure. What the person does there (a rummage, a cast, a swing, a paw into
// the hedge, a dig) is heard where they do it.

/// What a rummage in the glade is heard doing.
pub(super) fn rummaged(event: &crate::woods::rummage::Event) -> Option<Cue> {
    matches!(event, crate::woods::rummage::Event::Got { .. }).then_some(Cue::Find)
}

/// What a fishing trip is heard doing.
pub(super) fn angled(event: &crate::fishing::angling::Event) -> Option<Cue> {
    use crate::fishing::angling::Event;
    match event {
        Event::Spooked { .. } | Event::Hooked { .. } => Some(Cue::Splash),
        Event::Landed { .. } => Some(Cue::Find),
        _ => None,
    }
}

/// What a bug hunt is heard doing.
pub(super) fn hunted(event: &crate::meadow::catching::Event) -> Option<Cue> {
    use crate::meadow::catching::Event;
    match event {
        Event::Startled { .. } => Some(Cue::Flutter),
        Event::Caught { .. } | Event::Netted { .. } => Some(Cue::Find),
        _ => None,
    }
}

/// What a foray along the hedgerow is heard doing.
pub(super) fn foraged(event: &crate::hedgerow::foraging::Event) -> Option<Cue> {
    use crate::hedgerow::foraging::Event;
    match event {
        Event::Picked { .. } | Event::Saved { .. } => Some(Cue::Pick),
        _ => None,
    }
}

/// What a scavenge along the old track is heard doing: things lifted off a heap, a heap coming
/// down, and anything found in it.
pub(super) fn scavenged(event: &crate::track::scavenging::Event) -> Option<Cue> {
    use crate::track::scavenging::Event;
    match event {
        Event::Lifted { .. } | Event::Together { .. } => Some(Cue::Rummage),
        Event::Tumbled { .. } => Some(Cue::Thunk),
        Event::Got { .. } | Event::Map { .. } => Some(Cue::Find),
        _ => None,
    }
}

/// What a treasure hunt is heard doing: the heaps down a wrong turn, and the chest.
pub(super) fn treasure_heard(event: &crate::track::treasure::Event) -> Option<Cue> {
    use crate::track::treasure::Event;
    match event {
        Event::Heap(event) => scavenged(event),
        Event::Treasure { .. } => Some(Cue::Treasure),
        Event::Also { .. } => Some(Cue::Find),
        _ => None,
    }
}

/// What an expedition is heard doing: each leg as that place's own outing is, the far place found,
/// and whatever comes down the falls.
pub(super) fn expedition_heard(event: &crate::expedition::Event) -> Option<Cue> {
    use crate::expedition::Event;
    use crate::expedition::legs::LegEvent;
    match event {
        Event::Found { .. } => Some(Cue::Find),
        Event::Leg(LegEvent::Rummage(event)) => rummaged(event),
        Event::Leg(LegEvent::Fish(event)) => angled(event),
        Event::Leg(LegEvent::Bugs(event)) => hunted(event),
        Event::Leg(LegEvent::Forage(event)) => foraged(event),
        Event::Leg(LegEvent::Falls(crate::falls::wading::Event::Caught { .. })) => Some(Cue::Find),
        _ => None,
    }
}

/// How far the brush moves over someone, in scene pixels, for each stroke heard.
const STROKE: f32 = 10.0;

/// What has been heard so far, so each thing is heard once.
#[derive(Debug, Default)]
pub struct Listening {
    /// How far into the visit the train has been listened to.
    train: f32,
    /// The sack race's hops, the rope's heaves and the hammer taps heard so far.
    hops: u32,
    heaves: u32,
    taps: u32,
    /// What a story, or the clearing, was showing when last looked at.
    page: Option<Page>,
    /// How far the brush has moved since the last stroke was heard.
    pub brushed: f32,
    /// Whether the Sound window is open.
    window: bool,
}

/// What a story shows at a moment: a line, choices to make, or neither while it plays on.
#[derive(Clone, Debug, PartialEq)]
enum Page {
    Line(String),
    Choosing(usize),
    Between,
}

impl Page {
    fn of(line: Option<&str>, choices: usize) -> Self {
        match (line, choices) {
            (Some(text), _) => Page::Line(text.to_owned()),
            (None, 0) => Page::Between,
            (None, choices) => Page::Choosing(choices),
        }
    }
}

/// What is heard as a story goes from showing `before` to showing `now`: a page turned as it
/// opens or reads on past a line, a choice as one is made, and nothing as it plays on by itself.
fn turned(before: Option<&Page>, now: Option<&Page>) -> Option<Cue> {
    match (before, now?) {
        (Some(Page::Choosing(_)), Page::Choosing(_)) => None,
        (Some(Page::Choosing(_)), _) => Some(Cue::Choice),
        (None, _) => Some(Cue::Page),
        (Some(Page::Line(was)), now) if *now != Page::Line(was.clone()) => Some(Cue::Page),
        _ => None,
    }
}

/// What a pat, a snack, a toy or the brush sounds like, held out.
pub(super) fn offered(offer: Offer) -> Cue {
    match offer {
        Offer::Pet => Cue::Pat,
        Offer::Snack => Cue::Snack,
        Offer::Toy => Cue::Toy,
        Offer::Brush => Cue::Brush,
    }
}

/// How many hammer taps are heard in `worked` seconds of building: three quick ones and a
/// breath, over and over, the first as the work begins.
fn taps(worked: f32) -> u32 {
    const ROUND: f32 = 0.9;
    const GAP: f32 = 0.2;
    let rounds = (worked.max(0.0) / ROUND) as u32;
    let into = worked - rounds as f32 * ROUND;
    rounds * 3 + ((into / GAP) as u32 + 1).min(3)
}

impl HillApp {
    /// The piece for wherever the person is: each place's own by day, and its night piece once
    /// the lamps are lit. Quiet while the train comes and goes, its own sounds being enough, and
    /// once the clearing's business is over.
    pub(super) fn wanted_music(&self) -> Option<Music> {
        let track = match self.area {
            Area::Station if !self.station.is_settled() => return None,
            Area::Station | Area::Green => Track::Pastoral,
            Area::Clubhouse => Track::Fireside,
            Area::Fairground => Track::Waltz,
            Area::Woods => Track::Woods,
            Area::Hilltop => Track::Hilltop,
            Area::Clearing => match &self.clearing {
                Some((_, sovereign)) if sovereign.state() != State::Over => Track::Secret,
                _ => return None,
            },
        };
        Some(Music::new(track, self.daylight.lamps() >= 0.5))
    }

    /// Listens to everything that is heard without telling the window it has happened: the
    /// train on its timeline, the hops of the sack race, the heaves of the rope, the hammers on
    /// the Hilltop, and the pages and choices of a story.
    pub(super) fn listen(&mut self, now: f32) {
        self.sound.music(self.wanted_music());
        for heard in self.station.heard(self.listening.train, now) {
            match heard {
                Heard::Whistle => self.sound.play(Cue::Whistle),
                Heard::Chuff { near } => self.sound.play_softly(Cue::Chuff, near),
                Heard::Brakes => self.sound.play(Cue::Brakes),
            }
        }
        self.listening.train = now;
        let (hops, heaves) = match &self.fairground {
            Some((_, games)) => (games.sack_race.hops(), games.tug_of_war.heaves(now)),
            None => (0, 0),
        };
        if hops > self.listening.hops {
            self.sound.play(Cue::Hop);
        }
        if heaves > self.listening.heaves {
            self.sound.play(Cue::Heave);
        }
        (self.listening.hops, self.listening.heaves) = (hops, heaves);
        let taps = self
            .crafting
            .building
            .as_ref()
            .and_then(|building| building.worked_for(now))
            .map_or(0, taps);
        if taps > self.listening.taps {
            self.sound.play(Cue::Hammer);
        }
        self.listening.taps = taps;
        self.turn_pages();
    }

    /// A page turned as a story, or the clearing, reads on, and a choice made when one is.
    fn turn_pages(&mut self) {
        let page = match (self.area, &self.story, &self.clearing) {
            (Area::Clubhouse, Some((_, director)), _) => Some(Page::of(
                director.shown().map(|shown| shown.text.as_str()),
                director.choices().len(),
            )),
            (Area::Clearing, _, Some((_, sovereign))) => Some(Page::of(
                sovereign.stage.line.as_ref().map(|(_, text)| text.as_str()),
                sovereign.choices().len(),
            )),
            _ => None,
        };
        if let Some(cue) = turned(self.listening.page.as_ref(), page.as_ref()) {
            self.sound.play(cue);
        }
        self.listening.page = page;
    }

    /// The brush heard once for every so much stroking.
    pub(super) fn hear_brushing(&mut self, stroke: f32) {
        self.listening.brushed += stroke;
        if self.listening.brushed >= STROKE {
            self.listening.brushed = 0.0;
            self.sound.play(Cue::Brush);
        }
    }

    /// Mutes everything, or brings it back, and says which.
    pub(super) fn toggle_mute(&mut self, now: f32) {
        let muted = self.sound.toggle_mute();
        let line = if muted {
            "Sound off. Press M to bring it back."
        } else {
            self.sound.play(Cue::Click);
            "Sound on."
        };
        self.notice = Some((line.to_owned(), now));
    }

    /// The speaker in every bar, for the Sound window.
    pub(super) fn sound_button(&mut self, ui: &mut egui::Ui) {
        let muted = self.sound.levels().muted;
        let (icon, hint) = if muted {
            ("\u{1f507}", "Sound\u{2026} Muted: M brings it back")
        } else {
            ("\u{1f50a}", "Sound\u{2026} M mutes it")
        };
        if ui
            .selectable_label(self.listening.window, icon)
            .on_hover_text(hint)
            .clicked()
        {
            self.listening.window = !self.listening.window;
        }
    }

    /// The Sound window: how loud everything is, the music and the sounds, and mute. Changes are
    /// heard at once, and kept once a slider is let go.
    pub(super) fn sound_window(&mut self, ctx: &egui::Context) {
        if !self.listening.window {
            return;
        }
        let before = self.sound.levels();
        let mut levels = before;
        let mut open = true;
        let mut dragging = false;
        // Whether the master or the sounds' level was let go, to be heard at its new level.
        let mut set = false;
        egui::Window::new("Sound")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(260.0)
            .show(ctx, |ui| {
                egui::Grid::new("levels")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        // The music is heard at its new level as it is moved; the others
                        // with a click once they are let go.
                        for (label, level, clicks) in [
                            ("Master", &mut levels.master, true),
                            ("Music", &mut levels.music, false),
                            ("Sounds", &mut levels.sounds, true),
                        ] {
                            ui.label(label);
                            let mut percent = (*level * 100.0).round() as u32;
                            let slider = egui::Slider::new(&mut percent, 0..=100).suffix("%");
                            let response = ui.add_enabled(!levels.muted, slider);
                            if response.changed() {
                                *level = percent as f32 / 100.0;
                            }
                            dragging |= response.dragged();
                            let let_go = response.drag_stopped()
                                || response.changed() && !response.dragged();
                            set |= clicks && let_go;
                            ui.end_row();
                        }
                    });
                ui.add_space(4.0);
                ui.checkbox(&mut levels.muted, "Mute  M");
                if self.sound.silent() {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "There is nothing here to play sound on, so the Hill is quiet.",
                        )
                        .italics()
                        .small(),
                    );
                }
            });
        if levels != before {
            self.sound.set_levels(levels);
        }
        // A quiet click as a level is let go, to hear it at; and as the sound comes back on.
        if set || (before.muted && !levels.muted) {
            self.sound.play(Cue::Click);
        }
        if !dragging {
            self.sound.keep();
        }
        self.listening.window = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every one of the Woods' own cues is heard from something its outing tells the window, or
    /// played where the person acts; and an expedition's legs sound as their places' outings do.
    #[test]
    fn the_woods_are_heard_from_what_each_outing_tells() {
        use crate::expedition::{Event as Day, legs::LegEvent};
        use crate::track::heap::{How, Stuff};
        use crate::{
            fishing::angling, hedgerow::foraging, meadow::catching, track, woods::rummage,
        };
        let got = rummage::Event::Got {
            find: "smooth_pebble",
            spot: 0,
        };
        assert_eq!(rummaged(&got), Some(Cue::Find));
        assert_eq!(rummaged(&rummage::Event::Glint), None);
        assert_eq!(
            angled(&angling::Event::Hooked { fish: "trout" }),
            Some(Cue::Splash)
        );
        assert_eq!(
            hunted(&catching::Event::Startled { bug: "butterfly" }),
            Some(Cue::Flutter)
        );
        assert_eq!(
            foraged(&foraging::Event::Picked {
                find: "hazelnut",
                item: 0
            }),
            Some(Cue::Pick)
        );
        let lifted = track::scavenging::Event::Lifted {
            who: 1,
            stuff: Stuff::Plank,
            how: How::Plain,
        };
        assert_eq!(scavenged(&lifted), Some(Cue::Rummage));
        assert_eq!(
            treasure_heard(&track::treasure::Event::Treasure { find: "golden_axe" }),
            Some(Cue::Treasure)
        );
        assert_eq!(
            treasure_heard(&track::treasure::Event::Heap(lifted)),
            Some(Cue::Rummage),
            "a heap down a wrong turn sounds as one along the track"
        );
        assert_eq!(
            expedition_heard(&Day::Leg(LegEvent::Rummage(got))),
            Some(Cue::Find),
            "a leg sounds as its place's outing does"
        );
    }

    #[test]
    fn the_hammers_tap_three_times_and_take_a_breath() {
        assert_eq!(taps(0.0), 1, "the first tap is as the work begins");
        assert_eq!(taps(0.45), 3);
        assert_eq!(taps(0.85), 3, "a breath");
        assert_eq!(taps(0.9), 4);
        assert_eq!(taps(3.2), 12);
        assert!((0..200).map(|step| taps(step as f32 * 0.02)).is_sorted());
    }

    #[test]
    fn every_offer_is_heard_as_itself() {
        let cues: std::collections::BTreeSet<&str> =
            [Offer::Pet, Offer::Snack, Offer::Toy, Offer::Brush]
                .into_iter()
                .map(|offer| offered(offer).name())
                .collect();
        assert_eq!(cues.len(), 4);
    }

    #[test]
    fn a_page_is_turned_as_a_story_reads_on_and_a_choice_made_as_one_is() {
        let line = |text: &str| Some(Page::of(Some(text), 0));
        let between = Some(Page::of(None, 0));
        let choosing = Some(Page::of(None, 2));
        let heard =
            |before: &Option<Page>, now: &Option<Page>| turned(before.as_ref(), now.as_ref());
        assert_eq!(
            heard(&None, &line("Once")),
            Some(Cue::Page),
            "the story opens"
        );
        assert_eq!(heard(&line("Once"), &line("upon")), Some(Cue::Page));
        assert_eq!(heard(&line("Once"), &between), Some(Cue::Page));
        assert_eq!(heard(&line("Once"), &line("Once")), None, "still reading");
        assert_eq!(
            heard(&between, &line("a time")),
            None,
            "it played on by itself"
        );
        assert_eq!(heard(&line("Which?"), &choosing), Some(Cue::Page));
        assert_eq!(heard(&choosing, &choosing), None, "still choosing");
        assert_eq!(heard(&choosing, &between), Some(Cue::Choice));
        assert_eq!(heard(&choosing, &line("So be it")), Some(Cue::Choice));
        assert_eq!(heard(&line("The end."), &None), None, "the story is over");
    }
}
