use serde::Serialize;
use serde_json::Value;

use crate::autoplay::action::{ActionCandidate, action_reference};
use crate::locales::Locale;
use crate::prompt::builder::position_label;
use crate::state::NormalizedState;

use super::projection::{card_value, potion_value, relic_value};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PromptActionCandidate {
    #[serde(rename = "ref")]
    action_ref: String,
    kind: String,
    pub(crate) label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    choice_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hand_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    potion_slot: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    option: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reward_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    card: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    potion: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relic: Option<Value>,
}

pub(in crate::autoplay::planner::prompting) fn prompt_action_candidates(
    candidates: &[ActionCandidate],
    state: &NormalizedState,
    locale: &Locale,
) -> Vec<PromptActionCandidate> {
    candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let mut prompt = PromptActionCandidate {
                action_ref: action_reference(index),
                kind: candidate.kind.clone(),
                label: candidate.label.clone(),
                target_required: candidate.target_required,
                choice_index: None,
                hand_index: None,
                potion_slot: None,
                option: None,
                reward_kind: None,
                baseline: None,
                card: None,
                potion: None,
                relic: None,
            };
            add_source(&mut prompt, candidate, state, locale);
            prompt
        })
        .collect()
}

pub(in crate::autoplay::planner::prompting) fn annotate_map_action_positions(
    actions: &mut [PromptActionCandidate],
    locale: &Locale,
) {
    let total = actions.len();
    for (index, action) in actions.iter_mut().enumerate() {
        let position = position_label(index, total, locale);
        action.label = format!("({position}) {}", action.label);
    }
}

fn add_source(
    prompt: &mut PromptActionCandidate,
    candidate: &ActionCandidate,
    state: &NormalizedState,
    locale: &Locale,
) {
    let action_id = candidate.action_id.as_str();
    if matches!(action_id, "card_reward:skip" | "boss_card_reward:skip") {
        prompt.baseline = Some(true);
        return;
    }
    if let Some(identity) = action_id.strip_prefix("combat:play:") {
        if let Some((index, card)) = state
            .hand
            .iter()
            .enumerate()
            .find(|(_, card)| card.uuid.as_deref() == Some(identity))
        {
            prompt.hand_index = Some(index);
            prompt.card = Some(card_value(card, locale));
        }
        return;
    }
    if let Some(slot) = parse_index(action_id, "combat:potion:") {
        prompt.potion_slot = Some(slot);
        prompt.potion = state
            .potions
            .iter()
            .find(|potion| potion.slot == slot)
            .map(|potion| potion_value(potion, locale));
        return;
    }
    if let Some(index) = parse_index(action_id, "boss_card_reward:")
        .or_else(|| parse_index(action_id, "card_reward:"))
    {
        prompt.choice_index = Some(index);
        prompt.card = state
            .card_reward_choices
            .get(index)
            .map(|card| card_value(card, locale));
        return;
    }
    if let Some(index) = parse_index(action_id, "boss_relic:") {
        prompt.choice_index = Some(index);
        prompt.relic = state
            .boss_relic_choices
            .get(index)
            .map(|relic| relic_value(relic, locale));
        return;
    }
    if let Some(index) = parse_index(action_id, "hand_select:") {
        prompt.choice_index = Some(index);
        prompt.card = state.hand.get(index).map(|card| card_value(card, locale));
        return;
    }
    if let Some(index) = parse_index(action_id, "grid:") {
        prompt.choice_index = Some(index);
        let cards = if state.grid_cards.is_empty() {
            &state.hand
        } else {
            &state.grid_cards
        };
        prompt.card = cards.get(index).map(|card| card_value(card, locale));
        return;
    }
    for prefix in ["event:", "map:choice:", "shop:choice:", "chest:"] {
        if let Some(index) = parse_index(action_id, prefix) {
            prompt.choice_index = Some(index);
            return;
        }
    }
    if let Some(option) = action_id
        .strip_prefix("rest:")
        .filter(|option| *option != "proceed")
    {
        prompt.option = Some(option.to_string());
        return;
    }
    if let Some(reward) = action_id.strip_prefix("combat_reward:")
        && let Some((kind, index)) = reward.rsplit_once(':')
        && let Ok(index) = index.parse()
    {
        prompt.reward_kind = Some(kind.to_string());
        prompt.choice_index = Some(index);
    }
}

fn parse_index(action_id: &str, prefix: &str) -> Option<usize> {
    action_id.strip_prefix(prefix)?.parse().ok()
}
