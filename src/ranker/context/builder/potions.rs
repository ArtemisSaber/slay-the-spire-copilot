use crate::ranker::parser;
use crate::state::NormalizedState;

use super::super::variables;
use super::super::{ActionContext, ActionType};
use super::ContextInputs;

pub(super) fn build(state: &NormalizedState, inputs: &ContextInputs) -> Vec<ActionContext> {
    let mut contexts = Vec::new();
    for potion in state.potions.iter().filter(|potion| potion.can_use) {
        let parsed = parser::parse_description(&potion.description);
        if potion.requires_target {
            for monster in &state.monsters {
                let mut vars = inputs.action_vars(state, &parsed, 0, inputs.energy);
                variables::fill_target_vars(&mut vars, monster);
                contexts.push(ActionContext {
                    action_type: ActionType::UsePotion {
                        potion_name: potion.name.clone(),
                    },
                    card: None,
                    target_index: Some(monster.index),
                    target: Some(monster.clone()),
                    monsters: state.monsters.clone(),
                    parsed: parsed.clone(),
                    vars,
                });
            }
        } else {
            contexts.push(ActionContext {
                action_type: ActionType::UsePotion {
                    potion_name: potion.name.clone(),
                },
                card: None,
                target_index: None,
                target: None,
                monsters: state.monsters.clone(),
                parsed: parsed.clone(),
                vars: inputs.action_vars(state, &parsed, 0, inputs.energy),
            });
        }
    }
    contexts
}
