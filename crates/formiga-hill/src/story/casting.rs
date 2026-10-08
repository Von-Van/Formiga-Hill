//! Who plays whom: each role is cast, in order, by the first of its selectors that finds someone
//! in this colony. The same colony and the same seed always cast the same way.

use super::script::{Selector, Story};
use crate::cast::{Cast, Id};
use crate::dice::Dice;

/// Each role's player, by role, or `None` for an optional role nobody fitted.
pub fn cast_roles(story: &Story, cast: &Cast, seed: u64) -> Result<Vec<Option<Id>>, String> {
    if cast.members.len() < story.min_cast {
        return Err(format!(
            "\"{}\" needs at least {} travellers",
            story.title, story.min_cast
        ));
    }
    let mut dice = Dice::new(seed);
    let mut chosen: Vec<Option<Id>> = Vec::new();
    for role in &story.roles {
        let free: Vec<Id> = cast
            .ids()
            .filter(|id| !chosen.contains(&Some(*id)))
            .collect();
        let pick = role
            .select
            .iter()
            .find_map(|selector| choose(selector, &free, &chosen, cast, &mut dice));
        if pick.is_none() && !role.optional {
            return Err(format!(
                "nobody could play {} in \"{}\"",
                role.name, story.title
            ));
        }
        chosen.push(pick);
    }
    Ok(chosen)
}

fn choose(
    selector: &Selector,
    free: &[Id],
    chosen: &[Option<Id>],
    cast: &Cast,
    dice: &mut Dice,
) -> Option<Id> {
    let member = |id: &Id| cast.member(*id);
    let first = |test: &dyn Fn(Id) -> bool| free.iter().copied().find(|id| test(*id));
    let player = |role: usize| chosen.get(role).copied().flatten();
    match selector {
        Selector::Any => free.first().copied(),
        Selector::Random => {
            (!free.is_empty()).then(|| free[(dice.next_u64() % free.len() as u64) as usize])
        }
        Selector::Most(axis) | Selector::Least(axis) => {
            let value = |id: &Id| member(id).map_or(0.0, |member| axis.of(&member.axes()));
            let most = matches!(selector, Selector::Most(_));
            // Ties go to whoever comes first in the colony.
            free.iter().copied().reduce(|best, id| {
                let better = if most {
                    value(&id) > value(&best)
                } else {
                    value(&id) < value(&best)
                };
                if better { id } else { best }
            })
        }
        Selector::Kind(kind) => first(&|id| member(&id).is_some_and(|m| m.kind() == *kind)),
        Selector::Trait(wanted) => first(&|id| member(&id).is_some_and(|m| m.has_trait(*wanted))),
        Selector::Habit(habit) => first(&|id| member(&id).is_some_and(|m| m.has_habit(*habit))),
        Selector::FriendOf(role) => cast.closest_friend(player(*role)?, free.iter().copied()),
        Selector::PlaymateOf(role) => cast.playmate(player(*role)?, free.iter().copied()),
        Selector::RivalOf(role) => {
            let other = player(*role)?;
            first(&|id| cast.at_odds(other, id))
        }
        Selector::ParentOf(role) => {
            let parent = member(&player(*role)?)?.parent()?;
            free.contains(&parent).then_some(parent)
        }
        Selector::MiniOf(role) => {
            let parent = player(*role)?;
            first(&|id| member(&id).is_some_and(|m| m.parent() == Some(parent)))
        }
        Selector::Mini => first(&|id| member(&id).is_some_and(|m| m.parent().is_some())),
        Selector::Adult => first(&|id| member(&id).is_some_and(|m| m.parent().is_none())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cast::sample;
    use crate::story::lines::Lines;
    use crate::story::script::parse_story;

    /// A trait nobody in the sample colony has, for a selector that finds no one.
    fn nobodys_trait(cast: &Cast) -> String {
        let absent = formiga_travel::Trait::ALL
            .iter()
            .copied()
            .find(|wanted| !cast.members.iter().any(|member| member.has_trait(*wanted)))
            .unwrap();
        crate::cast::trait_id(absent)
    }

    fn story(roles: &str) -> Story {
        let text = format!(
            "[story]\nid = \"t\"\ntitle = \"title\"\narea = \"clubhouse\"\nstart = \"one\"\n{roles}\n[[scenes]]\nid = \"one\"\n"
        );
        parse_story(&text, &Lines::parse("title = \"Test\"").unwrap(), 1, 2).unwrap()
    }

    #[test]
    fn the_most_affectionate_plays_the_host() {
        let cast = sample();
        let story = story("[roles.host]\nselect = [\"most:affection\", \"any\"]");
        let host = cast_roles(&story, &cast, 1).unwrap()[0].unwrap();
        let fondest = cast
            .members
            .iter()
            .map(|member| member.axes().affection)
            .fold(f32::MIN, f32::max);
        assert_eq!(cast.member(host).unwrap().axes().affection, fondest);
    }

    #[test]
    fn nobody_plays_two_roles_and_fallbacks_fill_the_rest() {
        let cast = sample();
        let story = story(&format!(
            "[roles.a]\norder = 1\nselect = [\"any\"]\n[roles.b]\norder = 2\nselect = [\"kind:grump\", \"trait:{}\", \"any\"]",
            nobodys_trait(&cast)
        ));
        let ids = cast_roles(&story, &cast, 1).unwrap();
        assert!(ids[0].is_some() && ids[1].is_some());
        assert_ne!(ids[0], ids[1]);
    }

    #[test]
    fn an_optional_role_can_go_uncast_but_a_required_one_cannot() {
        let cast = sample();
        let optional = story(&format!(
            "[roles.ghost]\nselect = [\"trait:{}\"]\noptional = true",
            nobodys_trait(&cast)
        ));
        assert_eq!(cast_roles(&optional, &cast, 1).unwrap(), vec![None]);
        let mut too_big = optional.clone();
        too_big.min_cast = 99;
        assert!(cast_roles(&too_big, &cast, 1).is_err());
    }

    #[test]
    fn family_roles_find_family() {
        let cast = sample();
        let Some(mini) = cast.members.iter().find(|member| member.parent().is_some()) else {
            return;
        };
        let story = story(
            "[roles.little]\norder = 1\nselect = [\"mini\"]\noptional = true\n[roles.parent]\norder = 2\nselect = [\"parent_of:little\"]\noptional = true",
        );
        let ids = cast_roles(&story, &cast, 1).unwrap();
        let little = cast.member(ids[0].unwrap()).unwrap();
        assert_eq!(ids[1], little.parent());
        let _ = mini;
    }

    #[test]
    fn the_same_seed_casts_the_same_way() {
        let cast = sample();
        let story = story("[roles.someone]\nselect = [\"random\"]");
        assert_eq!(
            cast_roles(&story, &cast, 7).unwrap(),
            cast_roles(&story, &cast, 7).unwrap()
        );
    }
}
