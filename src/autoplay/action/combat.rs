use crate::autoplay::command_state::CommandState;
use crate::state::NormalizedState;

use super::{ActionCandidate, ActionRequest, AutoPlayAction, candidate, targeted_candidate};

pub(super) fn combat_candidates(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Vec<ActionCandidate> {
    let mut candidates = vec![];

    if command_state.has_command("play") {
        for card in &state.hand {
            if !card.playable {
                continue;
            }
            let Some(uuid) = card.uuid.as_deref() else {
                continue;
            };
            let action_id = format!("combat:play:{uuid}");
            let label = format!("Play {}", card.name);
            if card.has_target {
                candidates.push(targeted_candidate("play", action_id, label));
            } else {
                candidates.push(candidate("play", action_id, label));
            }
        }
    }

    if command_state.has_command("potion") {
        for potion in &state.potions {
            if !potion.can_use {
                continue;
            }
            let action_id = format!("combat:potion:{}", potion.slot);
            let label = format!("Drink {}", potion.name);
            if potion.requires_target {
                candidates.push(targeted_candidate("drink", action_id, label));
            } else {
                candidates.push(candidate("drink", action_id, label));
            }
        }
    }

    if command_state.has_command("end") {
        candidates.push(candidate(
            "end",
            "combat:end".to_string(),
            "End turn".to_string(),
        ));
    }

    candidates
}

pub(super) fn resolve_requested_combat(
    state: &NormalizedState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.action_id == "combat:end" && request.kind == "end" {
        return Some(AutoPlayAction::End);
    }

    if request.kind == "play" {
        let uuid = request.action_id.strip_prefix("combat:play:")?;
        let (hand_index, card) = state
            .hand
            .iter()
            .enumerate()
            .find(|(_, card)| card.uuid.as_deref() == Some(uuid))?;

        if card.cost > state.energy.unwrap_or(0) {
            return None;
        }

        let target_index = if card.has_target {
            let target_index = request.target_index?;
            if !state
                .monsters
                .iter()
                .any(|monster| monster.index == target_index)
            {
                return None;
            }
            Some(target_index)
        } else {
            None
        };

        return Some(AutoPlayAction::Play {
            hand_index,
            target_index,
        });
    }

    if request.kind == "drink" {
        let slot_index: usize = request
            .action_id
            .strip_prefix("combat:potion:")?
            .parse()
            .ok()?;
        let potion = state.potions.iter().find(|p| p.slot == slot_index)?;
        if !potion.can_use {
            return None;
        }

        let target_index = if potion.requires_target {
            let target = request.target_index?;
            if !state.monsters.iter().any(|monster| monster.index == target) {
                return None;
            }
            Some(target)
        } else {
            None
        };

        return Some(AutoPlayAction::Drink {
            slot_index,
            target_index,
        });
    }

    None
}
