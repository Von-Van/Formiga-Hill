//! What can stand on a Hilltop spot: a find as its own piece, a find planted and still growing,
//! or something built from a plan. Everything that draws or uses the Hilltop asks this, not the
//! finds, what a spot holds.

use crate::finds::plans::{self, Plan};
use crate::finds::{self, Use, art, growing};
use crate::paint::rgb;
use formiga_art::Rgba;
use serde::{Deserialize, Serialize};

/// What stands on one of the Hilltop's spots, or was lifted from one into the satchel to be
/// planted again as it is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Standing {
    /// A find, as its own full piece. Kept as the bare id, as everything on the Hilltop was before
    /// anything grew there: so a growing find kept this way is fully grown, and whatever stood on
    /// a colony's Hilltop before then stands there still, just as it did.
    Find(String),
    /// A find planted and still growing, `stage` stages along (see `finds::growing`).
    Planted { planted: String, stage: u8 },
    /// Something built from a plan (see `finds::plans`), out of finds that come back when it is
    /// taken apart.
    Built { built: String },
}

impl Standing {
    /// Everything that can stand on the Hilltop: every find's piece, every stage of everything
    /// that grows and everything that can be built. For checking that each fits any spot, and for
    /// reviewing them all.
    #[cfg(test)]
    pub fn every() -> Vec<Self> {
        let finds = finds::CATALOGUE
            .iter()
            .chain(finds::RELICS.iter())
            .map(|find| Self::from(find.id));
        let stages = growing::GROWING.iter().flat_map(|growth| {
            (0..growth.stages.len()).map(|stage| Self::Planted {
                planted: growth.id.to_owned(),
                stage: stage as u8,
            })
        });
        let built = plans::PLANS.iter().map(|plan| Self::built(plan.id));
        finds.chain(stages).chain(built).collect()
    }

    /// Something built from the plan with this id.
    pub fn built(plan: &str) -> Self {
        Self::Built {
            built: plan.to_owned(),
        }
    }

    /// The plan it was built from, if it was built.
    pub fn plan(&self) -> Option<&'static Plan> {
        match self {
            Self::Built { built } => plans::plan(built),
            _ => None,
        }
    }

    /// A find as it goes up from the satchel: planted, if it is one that grows.
    pub fn from_satchel(id: &str) -> Self {
        if growing::growth(id).is_some() {
            Self::Planted {
                planted: id.to_owned(),
                stage: 0,
            }
        } else {
            Self::Find(id.to_owned())
        }
    }

    /// The find it is, or the plan it was built from.
    pub fn id(&self) -> &str {
        match self {
            Self::Find(id) | Self::Planted { planted: id, .. } | Self::Built { built: id } => id,
        }
    }

    /// Whether Hill knows what it is. Anything it doesn't is kept, but neither drawn nor used.
    pub fn known(&self) -> bool {
        match self {
            Self::Built { built } => plans::plan(built).is_some(),
            _ => finds::find(self.id()).is_some(),
        }
    }

    /// The stage it has grown to, if it is still growing.
    pub fn stage(&self) -> Option<u8> {
        match self {
            Self::Planted { planted, stage } => growing::growth(planted)
                .filter(|growth| usize::from(*stage) < growth.stages.len())
                .map(|_| *stage),
            Self::Find(_) | Self::Built { .. } => None,
        }
    }

    /// Whether it is still growing: planted, and not yet its full piece.
    pub fn growing(&self) -> bool {
        self.stage().is_some()
    }

    /// Grows one stage, as a visit begins; after its last stage it is its full piece. Says
    /// whether it grew. Anything not growing stays just as it is.
    pub fn grow(&mut self) -> bool {
        if !self.growing() {
            return false;
        }
        let Self::Planted { planted, stage } = self else {
            return false;
        };
        let next = stage.saturating_add(1);
        if growing::growth(planted).is_some_and(|growth| usize::from(next) < growth.stages.len()) {
            *stage = next;
        } else {
            *self = Self::Find(std::mem::take(planted));
        }
        true
    }

    /// Whether, taken up, it goes back among the finds in the satchel: only a find that never
    /// grows. Anything planted is lifted whole, and keeps how far it has grown; anything built
    /// comes apart into its finds (see `comes_apart`).
    pub fn back_among_finds(&self) -> bool {
        matches!(self, Self::Find(id) if growing::growth(id).is_none())
    }

    /// What it comes apart into, if it was built: every find its plan took, and how many.
    pub fn comes_apart(&self) -> Option<&'static [(&'static str, u32)]> {
        self.plan().map(|plan| plan.needs)
    }

    /// What the person calls it: "A clump of bluebells", "Bluebells in bud", "The grand cairn".
    pub fn name(&self) -> &'static str {
        if let Self::Built { built } = self {
            return plans::plan(built).map_or("", |plan| plan.name);
        }
        let Some(find) = finds::find(self.id()) else {
            return "";
        };
        match (self.stage(), growing::growth(find.id)) {
            (Some(stage), Some(growth)) => growth.stages[usize::from(stage)],
            _ => find.piece,
        }
    }

    /// How the colony enjoys it. Anything still growing is looked after.
    pub fn use_(&self) -> Option<Use> {
        if self.growing() {
            return Some(Use::Tend);
        }
        match self {
            Self::Built { built } => plans::plan(built).map(|plan| plan.use_),
            _ => finds::find(self.id()).map(|find| find.use_),
        }
    }

    /// The finds it is made of: the one find, or every find its plan took.
    pub fn finds(&self) -> Vec<&'static str> {
        match self {
            Self::Built { built } => plans::plan(built).map(Plan::finds).unwrap_or_default(),
            _ => finds::find(self.id())
                .map(|find| vec![find.id])
                .unwrap_or_default(),
        }
    }

    /// Whether it is something to look up at the sky with, or made with one: the great telescope
    /// is as much for stargazing as the little one it was made from.
    pub fn sky_gazing(&self) -> bool {
        self.use_() == Some(Use::Gaze)
            || self
                .finds()
                .iter()
                .any(|id| finds::find(id).is_some_and(|find| find.use_ == Use::Gaze))
    }

    /// How it looks, standing on the Hilltop.
    pub fn piece(&self) -> art::Piece {
        match (self, self.stage()) {
            (Self::Built { built }, _) => art::built(built),
            (_, Some(stage)) => art::stage(self.id(), stage),
            (_, None) => art::piece(self.id()),
        }
    }

    /// What shines from it after dark, if anything: each light's colour, and where it shines from
    /// in `piece`, its own drawing.
    pub fn lights(&self, piece: &art::Piece) -> Vec<(Rgba, (i32, i32))> {
        let color = match self {
            Self::Find(id) => match id.as_str() {
                "lost_lantern" => rgb(0xffcf6a),
                "fallen_star" => rgb(0xfff2b0),
                "sovereign_arrow" => rgb(0xf4f6ff),
                _ => return Vec::new(),
            },
            Self::Planted { .. } => return Vec::new(),
            Self::Built { built } => return art::built_lights(built),
        };
        let middle = (
            piece.sprite.width() as i32 / 2,
            piece.sprite.height() as i32 / 2,
        );
        vec![(color, middle)]
    }
}

impl From<&str> for Standing {
    /// A find as its own full piece.
    fn from(id: &str) -> Self {
        Self::Find(id.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn something_planted_grows_a_stage_at_a_time_into_its_full_piece_and_no_further() {
        for growth in growing::GROWING {
            let mut planted = Standing::from_satchel(growth.id);
            assert_eq!(planted.stage(), Some(0), "{} is planted small", growth.id);
            let mut names = vec![planted.name()];
            for stage in 1..=growth.stages.len() {
                assert!(planted.grow(), "{} stopped at {stage}", growth.id);
                names.push(planted.name());
            }
            assert_eq!(planted, Standing::Find(growth.id.to_owned()));
            assert!(!planted.grow(), "grown is grown");
            assert_eq!(planted, Standing::Find(growth.id.to_owned()));
            let full = finds::find(growth.id).unwrap().piece;
            assert_eq!(names.last(), Some(&full));
            assert_eq!(&names[..growth.stages.len()], growth.stages);
        }
    }

    #[test]
    fn a_find_that_does_not_grow_stands_as_it_is() {
        let mut cairn = Standing::from_satchel("smooth_pebble");
        assert_eq!(cairn, Standing::from("smooth_pebble"));
        assert!(!cairn.grow() && !cairn.growing());
        assert!(cairn.back_among_finds());
        assert!(!Standing::from_satchel("bluebell_bulb").back_among_finds());
        assert!(
            !Standing::from("bluebell_bulb").back_among_finds(),
            "a grown clump is lifted whole, not turned back into a bulb"
        );
    }

    #[test]
    fn what_was_kept_before_anything_grew_reads_as_it_was_written() {
        let old: std::collections::BTreeMap<u8, Standing> =
            serde_json::from_str(r#"{"3": "bluebell_bulb", "9": "pinecone"}"#).unwrap();
        assert_eq!(old[&3], Standing::from("bluebell_bulb"));
        assert!(!old[&3].growing(), "it was standing, so it is grown");
        let planted = Standing::from_satchel("acorn_stash");
        let written = serde_json::to_string(&planted).unwrap();
        assert_eq!(written, r#"{"planted":"acorn_stash","stage":0}"#);
        assert_eq!(serde_json::from_str::<Standing>(&written).unwrap(), planted);
    }
}
