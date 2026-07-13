use std::collections::HashMap;

use crate::ranker::parser::ParsedEffects;
use crate::state::NormalizedState;

use super::variables;
use super::{ActionContext, ActionType};

mod cards;
mod potions;

pub(super) fn build_all(state: &NormalizedState) -> Vec<ActionContext> {
    let inputs = ContextInputs::from_state(state);
    tracing::debug!(
        "ranker building contexts hand={} potions={} mons={}",
        inputs.useful_count,
        state.potions.iter().filter(|potion| potion.can_use).count(),
        state.monsters.len(),
    );

    let mut contexts = cards::build(state, &inputs);
    contexts.extend(potions::build(state, &inputs));
    contexts.push(end_turn_context(state, &inputs));

    tracing::debug!("ranker built {} contexts", contexts.len());
    contexts
}

pub(super) struct ContextInputs {
    pub(super) energy: i64,
    pub(super) block: i64,
    pub(super) hp: i64,
    pub(super) incoming: i64,
    pub(super) monster_count: f64,
    pub(super) monsters_total: f64,
    pub(super) useful_count: f64,
    pub(super) chemical_x: f64,
}

impl ContextInputs {
    fn from_state(state: &NormalizedState) -> Self {
        Self {
            energy: state.energy.unwrap_or(0),
            block: state.block.unwrap_or(0),
            hp: state.current_hp.unwrap_or(0),
            incoming: state.incoming_damage,
            monster_count: state.monsters.len() as f64,
            monsters_total: state
                .monsters
                .iter()
                .map(|monster| monster.current_hp.unwrap_or(0) + monster.block.unwrap_or(0))
                .sum::<i64>() as f64,
            useful_count: state
                .hand
                .iter()
                .filter(|card| card.playable && card.uuid.is_some())
                .count() as f64,
            chemical_x: if state.relics.iter().any(|relic| relic.id == "Chemical X") {
                2.0
            } else {
                0.0
            },
        }
    }

    pub(super) fn action_vars(
        &self,
        state: &NormalizedState,
        parsed: &ParsedEffects,
        cost: i64,
        remaining_energy: i64,
    ) -> HashMap<String, f64> {
        let mut vars = variables::base_vars(self, state, remaining_energy);
        vars.insert("cost".to_string(), cost as f64);
        vars.insert("current_energy".to_string(), self.energy as f64);
        vars.insert("remaining_energy".to_string(), remaining_energy as f64);
        variables::fill_parsed_vars(&mut vars, parsed);
        vars
    }
}

fn end_turn_context(state: &NormalizedState, inputs: &ContextInputs) -> ActionContext {
    let parsed = ParsedEffects::default();
    ActionContext {
        action_type: ActionType::EndTurn,
        card: None,
        target_index: None,
        target: None,
        monsters: state.monsters.clone(),
        parsed: parsed.clone(),
        vars: inputs.action_vars(state, &parsed, 0, inputs.energy),
    }
}
