use std::collections::BTreeSet;

use super::derive_tags;
use crate::learning::action::SemanticAction;
use crate::learning::telemetry::RecordedRankedAction;
use crate::ranker::context::ActionType;
use crate::ranker::engine::ScoredAction;
use crate::state::NormalizedState;

pub fn recorded_ranked_actions(state: &NormalizedState) -> Vec<RecordedRankedAction> {
    if state.screen_type.as_ref().map(|screen| screen.as_str()) != Some("NONE") {
        return vec![];
    }
    crate::ranker::rank(state)
        .into_iter()
        .filter(|scored| !scored.is_avoid)
        .filter_map(|scored| recorded_action(state, scored))
        .collect()
}

pub fn ranker_tags(state: &NormalizedState) -> Vec<String> {
    crate::ranker::rank(state)
        .iter()
        .filter(|scored| !scored.is_avoid)
        .flat_map(|scored| derive_tags(&scored.breakdown))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn recorded_action(state: &NormalizedState, scored: ScoredAction) -> Option<RecordedRankedAction> {
    let semantic_action = match &scored.action_type {
        ActionType::PlayCard { card_id, .. } => {
            let card = state
                .hand
                .iter()
                .find(|card| card.uuid.as_deref() == Some(card_id))?;
            SemanticAction::PlayCard {
                card_id: valid_id(&card.id)?.to_string(),
                upgraded: card.upgraded,
                target_monster_id: stable_target(state, scored.target_index)?,
            }
        }
        ActionType::UsePotion { potion_name } => SemanticAction::UsePotion {
            potion_id: unique_potion_id(state, potion_name)?,
            target_monster_id: stable_target(state, scored.target_index)?,
        },
        ActionType::EndTurn => SemanticAction::EndTurn,
    };
    let mut tags = derive_tags(&scored.breakdown);
    tags.sort();
    tags.dedup();
    Some(RecordedRankedAction {
        semantic_action,
        score: scored.score,
        tags,
    })
}

fn unique_potion_id(state: &NormalizedState, potion_name: &str) -> Option<String> {
    let ids: BTreeSet<_> = state
        .potions
        .iter()
        .filter(|potion| potion.name == potion_name)
        .filter_map(|potion| potion.id.as_deref().and_then(valid_id))
        .collect();
    (ids.len() == 1).then(|| (*ids.first().expect("one value checked")).to_string())
}

fn stable_target(state: &NormalizedState, index: Option<usize>) -> Option<Option<String>> {
    let Some(index) = index else {
        return Some(None);
    };
    let id = state
        .monsters
        .iter()
        .find(|monster| monster.index == index)?
        .monster_id
        .as_deref()
        .and_then(valid_id)?;
    (state
        .monsters
        .iter()
        .filter(|monster| monster.monster_id.as_deref() == Some(id))
        .count()
        == 1)
        .then(|| Some(id.to_string()))
}

fn valid_id(id: &str) -> Option<&str> {
    let id = id.trim();
    (!id.is_empty() && id != "?" && id.len() <= 256).then_some(id)
}
