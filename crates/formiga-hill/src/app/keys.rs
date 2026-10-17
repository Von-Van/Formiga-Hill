//! The Hill from the keyboard. In the Woods, Left and Right step through whatever a click would
//! choose right now (a spot to search, where to cast, a bug, something ripe, a heap, a way at a
//! fork) and Enter chooses it, exactly as a click there would. The Keys card lists every key.
//!
//! What can be chosen is found by asking the Woods what is under the pointer all over the scene,
//! so it is always the same as what a click would find, for every activity, with nothing kept in
//! step by hand.

use super::{Area, Card, HillApp, paper};
use crate::station::{SCENE_HEIGHT, SCENE_WIDTH};
use eframe::egui;

/// How far apart, in scene pixels, the Woods is asked what is there.
const SAMPLE: usize = 3;

/// Points named the same and this close together are one thing.
const SAME_THING: f32 = 20.0;

/// Something the keyboard can choose: what it is called, and a point on it a click would hit.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Target {
    pub name: String,
    pub at: (f32, f32),
}

/// Gathers what `named` says is at each sampled point into things, left to right: points with
/// the same name close together are one thing, chosen at the sample nearest its middle.
pub(super) fn gather(
    width: u32,
    height: u32,
    named: impl Fn((f32, f32)) -> Option<String>,
) -> Vec<Target> {
    struct Cluster {
        name: String,
        points: Vec<(f32, f32)>,
    }
    let mut clusters: Vec<Cluster> = Vec::new();
    for y in (1..height as usize).step_by(SAMPLE) {
        for x in (1..width as usize).step_by(SAMPLE) {
            let at = (x as f32, y as f32);
            let Some(name) = named(at) else {
                continue;
            };
            let near = clusters.iter_mut().find(|cluster| {
                cluster.name == name
                    && cluster
                        .points
                        .iter()
                        .any(|p| crate::playground::distance(*p, at) <= SAME_THING)
            });
            match near {
                Some(cluster) => cluster.points.push(at),
                None => clusters.push(Cluster {
                    name,
                    points: vec![at],
                }),
            }
        }
    }
    let mut targets: Vec<Target> = clusters
        .into_iter()
        .map(|cluster| {
            let count = cluster.points.len() as f32;
            let middle = cluster
                .points
                .iter()
                .fold((0.0, 0.0), |(x, y), p| (x + p.0 / count, y + p.1 / count));
            let at = cluster
                .points
                .iter()
                .copied()
                .min_by(|a, b| {
                    crate::playground::distance(*a, middle)
                        .total_cmp(&crate::playground::distance(*b, middle))
                })
                .unwrap_or(middle);
            Target {
                name: cluster.name,
                at,
            }
        })
        .collect();
    targets.sort_by(|a, b| a.at.0.total_cmp(&b.at.0).then(a.at.1.total_cmp(&b.at.1)));
    targets
}

/// The next target along from `from` (or the first), `step` places on, going round.
pub(super) fn step(targets: &[Target], from: Option<&Target>, step: isize) -> Option<Target> {
    if targets.is_empty() {
        return None;
    }
    let len = targets.len() as isize;
    let index = match from.and_then(|from| nearest(targets, from)) {
        Some(index) => (index as isize + step).rem_euclid(len),
        None if step < 0 => len - 1,
        None => 0,
    };
    targets.get(index as usize).cloned()
}

/// Where `target` is among `targets` now: the same thing, as near where it was as can be, since
/// bugs fly and the party walks.
fn nearest(targets: &[Target], target: &Target) -> Option<usize> {
    let same = |t: &&Target| t.name == target.name;
    let near = |(_, t): &(usize, &Target)| crate::playground::distance(t.at, target.at);
    targets
        .iter()
        .enumerate()
        .filter(|(_, t)| same(t))
        .min_by(|a, b| near(a).total_cmp(&near(b)))
        .or_else(|| {
            targets
                .iter()
                .enumerate()
                .min_by(|a, b| near(a).total_cmp(&near(b)))
        })
        .map(|(index, _)| index)
}

impl HillApp {
    /// Everything in the Woods a click would choose right now. Nothing while the map editor is
    /// open: it has its own ways of being worked.
    fn woods_targets(&self) -> Vec<Target> {
        if self.area != Area::Woods || self.woods.making.is_some() {
            return Vec::new();
        }
        gather(SCENE_WIDTH, SCENE_HEIGHT, |at| {
            self.woods_hover(Some(at)).map(|(name, _)| name)
        })
    }

    /// Left and Right step through what can be chosen in the Woods, and Enter chooses it.
    pub(super) fn woods_keys(&mut self, input: &egui::InputState, now: f32) {
        if self.area != Area::Woods {
            self.aim = None;
            return;
        }
        let along = if input.key_pressed(egui::Key::ArrowRight) {
            1
        } else if input.key_pressed(egui::Key::ArrowLeft) {
            -1
        } else {
            0
        };
        if along != 0 {
            let targets = self.woods_targets();
            self.aim = step(&targets, self.aim.as_ref(), along);
            self.aim_mouse = self.mouse;
        }
        if input.key_pressed(egui::Key::Enter)
            && let Some(aim) = self.aim.clone()
        {
            // Bugs fly and the party walks: choose the same thing where it is now.
            let targets = self.woods_targets();
            let now_at = nearest(&targets, &aim).map(|index| targets[index].clone());
            if let Some(target) = now_at {
                self.woods_click(Some(target.at), now);
                self.aim = Some(target);
            }
        }
    }

    /// The pointer the Woods should see: the mouse's, unless the keyboard is choosing and the
    /// mouse has not moved or been pressed since, when it is the thing chosen.
    pub(super) fn keyboard_pointer(
        &mut self,
        mouse: Option<(f32, f32)>,
        pressed: bool,
    ) -> Option<(f32, f32)> {
        self.mouse = mouse;
        let Some(aim) = &self.aim else {
            return mouse;
        };
        if self.area != Area::Woods || pressed || mouse != self.aim_mouse {
            self.aim = None;
            return mouse;
        }
        Some(aim.at)
    }

    /// The Keys button among the buttons everywhere.
    pub(super) fn keys_button(&mut self, ui: &mut egui::Ui) {
        let response = ui.add(paper::Button::new("?").selected(self.showing(Card::Keys)));
        paper::name_it(ui, &response, "Keys  K");
        if response.clicked() {
            self.toggle(Card::Keys);
        }
    }

    /// Every key the Hill answers to, where it works.
    pub(super) fn keys_window(&mut self, ctx: &egui::Context) {
        if !self.showing(Card::Keys) {
            return;
        }
        self.show_card(ctx, Card::Keys, "Keys", 0.42, |_, ui| {
            for (index, (heading, keys)) in KEYS.iter().enumerate() {
                if index > 0 {
                    paper::rule(ui);
                }
                ui.add(paper::Words::new(*heading).color(paper::FOREST));
                for (key, what) in *keys {
                    paper::words(ui, format!("{key}: {what}"));
                }
            }
        });
    }
}

/// The Keys card: each place's keys, as the person would press them.
const KEYS: [(&str, &[(&str, &str)]); 6] = [
    (
        "Everywhere",
        &[
            ("C", "the camera, out or away"),
            ("M", "sound off or on"),
            ("K", "this card"),
            ("Esc", "close a card, or go back a step"),
            ("Space or Enter", "hurry the train in"),
        ],
    ),
    (
        "Out and about",
        &[("1 to 4", "hold out a pat, a snack, a toy or the brush")],
    ),
    ("At the Fairground", &[("Esc", "call a game off")]),
    ("On the Hilltop", &[("Esc", "put down whatever is held")]),
    (
        "In a story",
        &[("Space or Enter", "read on"), ("1 to 4", "make a choice")],
    ),
    (
        "In the Woods",
        &[
            ("Left or Right", "step through what can be chosen"),
            ("Enter", "choose it"),
            ("Space", "try, strike, swing the net, or hold to reel in"),
            ("Up or W", "hold to creep up on a bug"),
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn two_spots(at: (f32, f32)) -> Option<String> {
        let near = |x: f32, y: f32| crate::playground::distance(at, (x, y)) <= 6.0;
        if near(30.0, 40.0) {
            Some("Mossy log".to_owned())
        } else if near(90.0, 20.0) || near(150.0, 20.0) {
            Some("Fern".to_owned())
        } else {
            None
        }
    }

    #[test]
    fn what_a_click_would_choose_is_gathered_into_things_left_to_right() {
        let targets = gather(200, 60, two_spots);
        let names: Vec<&str> = targets.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["Mossy log", "Fern", "Fern"], "two ferns, apart");
        for target in &targets {
            assert_eq!(
                two_spots(target.at).as_deref(),
                Some(target.name.as_str()),
                "a click at {:?} finds it",
                target.at
            );
        }
    }

    #[test]
    fn stepping_goes_round_both_ways() {
        let targets = gather(200, 60, two_spots);
        let first = step(&targets, None, 1).unwrap();
        assert_eq!(first, targets[0]);
        assert_eq!(step(&targets, None, -1).unwrap(), targets[2]);
        assert_eq!(step(&targets, Some(&targets[2]), 1).unwrap(), targets[0]);
        assert_eq!(step(&targets, Some(&targets[0]), -1).unwrap(), targets[2]);
        assert!(step(&[], None, 1).is_none());
    }

    #[test]
    fn a_thing_that_has_moved_is_still_found() {
        let targets = gather(200, 60, two_spots);
        let moved = Target {
            name: "Mossy log".to_owned(),
            at: (60.0, 40.0),
        };
        assert_eq!(nearest(&targets, &moved), Some(0));
    }

    #[test]
    fn every_key_on_the_card_is_one_the_hill_answers_to() {
        let all: Vec<&str> = KEYS
            .iter()
            .flat_map(|(_, keys)| keys.iter().map(|(key, _)| *key))
            .collect();
        for key in ["C", "M", "K", "Esc", "Enter", "Space"] {
            assert!(all.iter().any(|k| k.contains(key)), "{key}");
        }
    }
}
