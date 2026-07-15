use crate::autoplay::action::{ActionCandidate, AutoPlayAction};
use crate::state::NormalizedState;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SemanticAction {
    PlayCard {
        card_id: String,
        upgraded: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        target_monster_id: Option<String>,
    },
    UsePotion {
        potion_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        target_monster_id: Option<String>,
    },
    EndTurn,
}

pub fn available_semantic_actions(
    candidates: &[ActionCandidate],
    state: &NormalizedState,
) -> Vec<SemanticAction> {
    let mut actions = Vec::new();
    for candidate in candidates {
        match candidate.kind.as_str() {
            "play" => add_card_actions(candidate, state, &mut actions),
            "drink" => add_potion_actions(candidate, state, &mut actions),
            "end" if candidate.action_id == "combat:end" => actions.push(SemanticAction::EndTurn),
            _ => {}
        }
    }
    let mut seen = HashSet::new();
    actions.retain(|action| seen.insert(action.clone()));
    actions
}

fn add_card_actions(
    candidate: &ActionCandidate,
    state: &NormalizedState,
    actions: &mut Vec<SemanticAction>,
) {
    let Some(uuid) = candidate.action_id.strip_prefix("combat:play:") else {
        return;
    };
    let Some(card) = state
        .hand
        .iter()
        .find(|card| card.uuid.as_deref() == Some(uuid))
    else {
        return;
    };
    let Some(card_id) = valid_id(&card.id) else {
        return;
    };
    for target_monster_id in stable_targets(candidate, state) {
        actions.push(SemanticAction::PlayCard {
            card_id: card_id.to_string(),
            upgraded: card.upgraded,
            target_monster_id,
        });
    }
}

fn add_potion_actions(
    candidate: &ActionCandidate,
    state: &NormalizedState,
    actions: &mut Vec<SemanticAction>,
) {
    let Some(slot) = candidate
        .action_id
        .strip_prefix("combat:potion:")
        .and_then(|slot| slot.parse::<usize>().ok())
    else {
        return;
    };
    let Some(potion_id) = state
        .potions
        .iter()
        .find(|potion| potion.slot == slot)
        .and_then(|potion| potion.id.as_deref())
        .and_then(valid_id)
    else {
        return;
    };
    for target_monster_id in stable_targets(candidate, state) {
        actions.push(SemanticAction::UsePotion {
            potion_id: potion_id.to_string(),
            target_monster_id,
        });
    }
}

fn stable_targets(candidate: &ActionCandidate, state: &NormalizedState) -> Vec<Option<String>> {
    if candidate.target_required != Some(true) {
        return vec![None];
    }
    state
        .monsters
        .iter()
        .filter(|monster| monster.current_hp.unwrap_or(1) > 0)
        .filter_map(|monster| unique_monster_id(state, monster.index))
        .map(|id| Some(id.to_string()))
        .collect()
}

impl SemanticAction {
    pub fn from_execution(action: &AutoPlayAction, state: &NormalizedState) -> Option<Self> {
        match action {
            AutoPlayAction::Play {
                hand_index,
                target_index,
            } => {
                let card = state.hand.get(*hand_index)?;
                valid_id(&card.id)?;
                Some(Self::PlayCard {
                    card_id: card.id.clone(),
                    upgraded: card.upgraded,
                    target_monster_id: target_id(state, *target_index)?,
                })
            }
            AutoPlayAction::Drink {
                slot_index,
                target_index,
            } => {
                let potion = state
                    .potions
                    .iter()
                    .find(|potion| potion.slot == *slot_index)?;
                let potion_id = potion.id.as_deref().and_then(valid_id)?.to_string();
                Some(Self::UsePotion {
                    potion_id,
                    target_monster_id: target_id(state, *target_index)?,
                })
            }
            AutoPlayAction::End => Some(Self::EndTurn),
            AutoPlayAction::Choose(_)
            | AutoPlayAction::Skip
            | AutoPlayAction::Proceed
            | AutoPlayAction::Leave => None,
        }
    }
}

fn target_id(state: &NormalizedState, target_index: Option<usize>) -> Option<Option<String>> {
    let Some(index) = target_index else {
        return Some(None);
    };
    unique_monster_id(state, index).map(|id| Some(id.to_string()))
}

fn unique_monster_id(state: &NormalizedState, index: usize) -> Option<&str> {
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
        .then_some(id)
}

fn valid_id(id: &str) -> Option<&str> {
    let trimmed = id.trim();
    (!trimmed.is_empty() && trimmed == id && id != "?" && id.len() <= 256).then_some(id)
}
