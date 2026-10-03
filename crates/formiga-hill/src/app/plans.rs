//! Building on the Hilltop in the window: the plans the colony has thought of, what it can build
//! now and what it nearly can, the colony building one on the spot the person chooses, and taking
//! something built apart again.

use super::HillApp;
use super::arranging::Placing;
use crate::finds;
use crate::finds::plans::{PLANS, Plan};
use crate::hilltop::building::{Building, Moment, PUFF_SECS, puff};
use eframe::egui;
use formiga_art::Canvas;

/// Building on the Hilltop, while there is any: whether the plans are open, something going up,
/// and puffs of dust still clearing where something was taken apart.
#[derive(Default)]
pub struct Crafting {
    pub open: bool,
    pub building: Option<Building>,
    puffs: Vec<(u8, f32)>,
}

/// "a smooth pebble", "2 smooth pebbles", from a find's name, "A smooth pebble".
fn some(id: &str, count: u32) -> String {
    let Some(find) = finds::find(id) else {
        return id.to_owned();
    };
    let name = find.name.to_lowercase();
    if count == 1 {
        return name;
    }
    let bare = strip_article(&name);
    let plural = match bare.split_once(' ') {
        // "piece of river glass", "message in a bottle": the first word takes the plural.
        Some((first, rest)) if ["piece", "message", "sprig"].contains(&first) => {
            format!("{first}s {rest}")
        }
        _ if ["s", "x", "ch", "sh"].iter().any(|end| bare.ends_with(end)) => format!("{bare}es"),
        _ if bare.ends_with('y') && !bare.ends_with("ey") => {
            format!("{}ies", &bare[..bare.len() - 1])
        }
        _ => format!("{bare}s"),
    };
    format!("{count} {plural}")
}

/// What a plan takes, as a list: "3 smooth pebbles", "a brass lens and a lost lantern".
pub(super) fn takes(plan: &Plan) -> String {
    let parts: Vec<String> = plan
        .needs
        .iter()
        .map(|&(id, count)| some(id, count))
        .collect();
    listed(&parts)
}

/// What a plan is still short of, gently: "needs one more smooth pebble". A find not yet found is
/// only hinted at, by where it is found.
pub(super) fn short_of(
    plan: &Plan,
    satchel: &std::collections::BTreeMap<String, u32>,
    found: impl Fn(&str) -> bool,
) -> String {
    let parts: Vec<String> = plan
        .short(satchel)
        .into_iter()
        .map(|(id, count)| match finds::find(id) {
            Some(find) if !found(id) => format!(
                "something {} not found yet, {}",
                find.tier.label(),
                find.kind.whereabouts()
            ),
            _ if count == 1 && satchel.get(id).is_some_and(|have| *have > 0) => {
                format!("one more {}", strip_article(&some(id, 1)))
            }
            _ => some(id, count),
        })
        .collect();
    format!("Needs {}.", listed(&parts))
}

/// "smooth pebble", from "a smooth pebble".
fn strip_article(name: &str) -> &str {
    ["a ", "an ", "the "]
        .iter()
        .find_map(|article| name.strip_prefix(article))
        .unwrap_or(name)
}

fn listed(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

impl HillApp {
    /// The plans the colony has thought of: those with at least one of their finds in the journal.
    /// Homelier things first, landmarks last.
    pub(super) fn thought_of(&self) -> Vec<&'static Plan> {
        let colony = self.memories.colony();
        let mut plans: Vec<&'static Plan> = PLANS
            .iter()
            .filter(|plan| plan.thought_of(|id| colony.finds.contains_key(id)))
            .collect();
        plans.sort_by_key(|plan| plan.tier());
        plans
    }

    /// The plans there is enough in the satchel to build now.
    pub(super) fn ready_to_build(&self) -> Vec<&'static Plan> {
        let satchel = &self.memories.colony().satchel;
        self.thought_of()
            .into_iter()
            .filter(|plan| plan.ready(satchel))
            .collect()
    }

    /// Builds a plan on a spot: the finds come out of the satchel at once, and the colony gathers
    /// round to build it, the piece appearing when they are done.
    pub(super) fn start_building(&mut self, spot: u8, plan: &str, now: f32) {
        if self.crafting.building.is_some() || !self.memories.build(spot, plan) {
            return;
        }
        if let Some(ground) = &mut self.hilltop {
            self.crafting.building = Some(Building::begin(ground, spot, now));
        }
        self.crafting.open = false;
        self.refresh_hilltop();
    }

    /// Takes apart what was built on a spot, every find going back into the satchel.
    pub(super) fn take_apart(&mut self, spot: u8, now: f32) {
        let Some(standing) = self.memories.take_apart(spot) else {
            return;
        };
        if let Some(plan) = standing.plan() {
            self.notice = Some((
                format!(
                    "{} taken apart: {} back in the satchel.",
                    plan.name,
                    takes(plan)
                ),
                now,
            ));
        }
        if !self.arrival.cast.reduce_motion() {
            self.crafting.puffs.push((spot, now));
        }
        self.refresh_hilltop();
    }

    /// The summit's free play, and the building, if anything is going up.
    pub(super) fn tick_hilltop(&mut self, now: f32) {
        let Some(ground) = &mut self.hilltop else {
            return;
        };
        ground.tick(&self.arrival.cast, now);
        let moment = match &mut self.crafting.building {
            Some(building) => building.tick(ground, now),
            None => None,
        };
        match moment {
            Some(Moment::Appeared) => {
                let built = self
                    .crafting
                    .building
                    .as_ref()
                    .map(|building| building.spot);
                let name = built
                    .and_then(|spot| self.memories.colony().hilltop.get(&spot))
                    .map(|standing| standing.name().to_lowercase());
                if let Some(name) = name {
                    self.notice = Some((format!("The colony built {name}!"), now));
                }
                self.refresh_hilltop();
            }
            Some(Moment::Over) => self.crafting.building = None,
            None => {}
        }
        self.crafting
            .puffs
            .retain(|(_, since)| now - since < PUFF_SECS);
    }

    /// Puffs of dust where something has just appeared or come apart.
    pub(super) fn draw_crafting(&self, scene: &mut Canvas, now: f32) {
        if let Some(building) = &self.crafting.building {
            building.draw(scene, now);
        }
        for (spot, since) in &self.crafting.puffs {
            puff(scene, *spot, now - since);
        }
    }

    /// The spot being built on, while what is going up there isn't up yet.
    pub(super) fn going_up(&self) -> Option<u8> {
        self.crafting
            .building
            .as_ref()
            .filter(|building| !building.showing())
            .map(|building| building.spot)
    }

    /// The plans: what can be built now, with a button to build it; and what nearly can, with what
    /// it still needs. Only the plans the colony has thought of, so the list grows with the journal.
    pub(super) fn plans_window(&mut self, ctx: &egui::Context) {
        if !self.crafting.open || self.area != super::Area::Hilltop {
            return;
        }
        let colony = self.memories.colony();
        let found = |id: &str| colony.finds.contains_key(id);
        let thought_of = self.thought_of();
        let busy = self.crafting.building.is_some();
        let mut chosen = None;
        let mut open = true;
        egui::Window::new("Plans")
            .open(&mut open)
            .default_width(340.0)
            .default_height(380.0)
            .show(ctx, |ui| {
                if thought_of.is_empty() {
                    ui.label(
                        egui::RichText::new(
                            "Ideas for building come with what the Woods turns up.",
                        )
                        .italics(),
                    );
                    return;
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let (ready, nearly): (Vec<&Plan>, Vec<&Plan>) = thought_of
                        .iter()
                        .partition(|plan| plan.ready(&colony.satchel));
                    if !ready.is_empty() {
                        ui.strong("Ready to build");
                        for plan in ready {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(plan.name).strong());
                                let button = egui::Button::new("Build");
                                if ui.add_enabled(!busy, button).clicked() {
                                    chosen = Some(plan.id);
                                }
                            });
                            ui.label(egui::RichText::new(plan.blurb).small());
                            ui.label(
                                egui::RichText::new(format!("Takes {}", takes(plan)))
                                    .small()
                                    .italics(),
                            );
                        }
                    }
                    if !nearly.is_empty() {
                        ui.add_space(8.0);
                        ui.strong("Ideas");
                        for plan in nearly {
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(plan.name).strong());
                            ui.label(egui::RichText::new(plan.blurb).small());
                            ui.label(
                                egui::RichText::new(short_of(plan, &colony.satchel, found))
                                    .small()
                                    .italics(),
                            );
                        }
                    }
                    let unthought = PLANS.len() - thought_of.len();
                    if unthought > 0 {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("More ideas will come with more finds.").weak(),
                        );
                    }
                });
            });
        self.crafting.open = open;
        if let Some(plan) = chosen {
            self.placing = Some(Placing::Plan(plan));
            self.crafting.open = false;
        }
    }

    /// The plans for the journal, beside the finds: those built, and the ideas. Nothing until the
    /// colony has thought of one.
    pub(super) fn journal_plans(&self, ui: &mut egui::Ui) {
        let colony = self.memories.colony();
        let thought_of = self.thought_of();
        if thought_of.is_empty() {
            return;
        }
        ui.add_space(6.0);
        ui.strong(format!(
            "Plans ({} of {} thought of)",
            thought_of.len(),
            PLANS.len()
        ));
        for plan in thought_of {
            let standing = colony
                .hilltop
                .values()
                .filter(|standing| standing.plan().is_some_and(|built| built.id == plan.id))
                .count();
            let line = match standing {
                0 if plan.ready(&colony.satchel) => {
                    format!("Takes {} \u{b7} ready to build", takes(plan))
                }
                0 => format!("Takes {}", takes(plan)),
                1 => format!("Takes {} \u{b7} standing on the Hilltop", takes(plan)),
                many => format!(
                    "Takes {} \u{b7} {many} standing on the Hilltop",
                    takes(plan)
                ),
            };
            ui.label(egui::RichText::new(plan.name).strong());
            ui.label(egui::RichText::new(plan.blurb).small());
            ui.label(egui::RichText::new(line).small().italics());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finds::plans;
    use std::collections::BTreeMap;

    #[test]
    fn what_a_plan_takes_reads_as_a_list() {
        assert_eq!(
            takes(plans::plan("grand_cairn").unwrap()),
            "3 smooth pebbles"
        );
        assert_eq!(
            takes(plans::plan("bandstand").unwrap()),
            "a music box, a lost lantern and 2 pieces of river glass"
        );
        assert_eq!(
            takes(plans::plan("lantern_tree").unwrap()),
            "a message in a bottle and 2 lost lanterns"
        );
    }

    #[test]
    fn what_a_plan_needs_is_a_gentle_hint_and_keeps_unfound_things_secret() {
        let cairn = plans::plan("grand_cairn").unwrap();
        let satchel = BTreeMap::from([("smooth_pebble".to_owned(), 2)]);
        assert_eq!(
            short_of(cairn, &satchel, |_| true),
            "Needs one more smooth pebble."
        );
        let house = plans::plan("burrow_house").unwrap();
        let hint = short_of(house, &BTreeMap::new(), |id| id == "pinecone");
        assert!(
            hint.contains("something exceptional not found yet, in the earth"),
            "{hint}"
        );
        assert!(
            !hint.contains("door"),
            "the door is a secret till it is found: {hint}"
        );
    }
}
