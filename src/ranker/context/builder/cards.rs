use crate::ranker::parser;
use crate::state::{CardInfo, NormalizedState};

use super::super::cost;
use super::super::variables;
use super::super::{ActionContext, ActionType};
use super::ContextInputs;

pub(super) fn build(state: &NormalizedState, inputs: &ContextInputs) -> Vec<ActionContext> {
    let mut contexts = Vec::new();
    for card in state
        .hand
        .iter()
        .filter(|card| card.uuid.is_some() && card.playable)
    {
        let mut parsed = parser::parse_description(&card.description);
        let is_aoe = variables::is_aoe(card);
        let (cost, hits, amount) = cost::resolve_x_cost(card, inputs.energy, inputs.chemical_x);
        let remaining_energy = inputs.energy - cost;
        if remaining_energy < 0 {
            continue;
        }
        if let Some(hits) = hits {
            parsed.hits = hits;
        }
        if let Some(amount) = amount {
            cost::apply_x_cost_amount(&mut parsed, amount);
        }

        if card.has_target && card.card_type == "ATTACK" {
            push_targeted_contexts(
                &mut contexts,
                card,
                &parsed,
                state,
                inputs,
                cost,
                remaining_energy,
                false,
            );
        } else if is_aoe {
            push_targeted_contexts(
                &mut contexts,
                card,
                &parsed,
                state,
                inputs,
                cost,
                remaining_energy,
                true,
            );
        } else {
            contexts.push(targetless_context(
                card,
                &parsed,
                state,
                inputs,
                cost,
                remaining_energy,
            ));
        }
    }
    contexts
}

#[allow(
    clippy::too_many_arguments,
    reason = "keeps target-context construction explicit"
)]
fn push_targeted_contexts(
    contexts: &mut Vec<ActionContext>,
    card: &CardInfo,
    parsed: &parser::ParsedEffects,
    state: &NormalizedState,
    inputs: &ContextInputs,
    cost: i64,
    remaining_energy: i64,
    aoe: bool,
) {
    for monster in &state.monsters {
        let mut vars = card_vars(card, parsed, state, inputs, cost, remaining_energy);
        variables::fill_target_vars(&mut vars, monster);
        if aoe {
            variables::fill_retaliatory_damage(&mut vars, &state.monsters);
        }
        contexts.push(ActionContext {
            action_type: play_action(card),
            card: Some(card.clone()),
            target_index: Some(monster.index),
            target: Some(monster.clone()),
            monsters: state.monsters.clone(),
            parsed: parsed.clone(),
            vars,
        });
    }
}

fn targetless_context(
    card: &CardInfo,
    parsed: &parser::ParsedEffects,
    state: &NormalizedState,
    inputs: &ContextInputs,
    cost: i64,
    remaining_energy: i64,
) -> ActionContext {
    ActionContext {
        action_type: play_action(card),
        card: Some(card.clone()),
        target_index: None,
        target: None,
        monsters: state.monsters.clone(),
        parsed: parsed.clone(),
        vars: card_vars(card, parsed, state, inputs, cost, remaining_energy),
    }
}

fn card_vars(
    card: &CardInfo,
    parsed: &parser::ParsedEffects,
    state: &NormalizedState,
    inputs: &ContextInputs,
    cost: i64,
    remaining_energy: i64,
) -> std::collections::HashMap<String, f64> {
    let mut vars = inputs.action_vars(state, parsed, cost, remaining_energy);
    variables::fill_card_meta_vars(&mut vars, card, state);
    vars
}

fn play_action(card: &CardInfo) -> ActionType {
    ActionType::PlayCard {
        card_id: card.uuid.clone().unwrap_or_else(|| card.id.clone()),
        card_name: card.name.clone(),
    }
}
