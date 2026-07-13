use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlayMode, AutoPlaySession, ControlLoad};
use crate::protocol;
use crate::state::NormalizedState;

mod combat;
mod navigation;
mod rewards;

use combat::{combat_candidates, resolve_requested_combat};
use navigation::{
    chest_candidates, event_candidates, grid_candidates, hand_select_candidates, map_candidates,
    resolve_requested_indexed, resolve_requested_shop, shop_candidates,
};
use rewards::{
    boss_reward_candidates, card_reward_candidates, combat_reward_candidates,
    resolve_requested_boss_reward, resolve_requested_card_reward, resolve_requested_combat_reward,
    resolve_requested_rest, rest_candidates,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoPlayAction {
    Choose(usize),
    Play {
        hand_index: usize,
        target_index: Option<usize>,
    },
    Drink {
        slot_index: usize,
        target_index: Option<usize>,
    },
    End,
    Skip,
    Proceed,
    Leave,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ActionRequest {
    pub kind: String,
    pub action_id: String,
    #[serde(default)]
    pub target_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionCandidate {
    pub kind: String,
    pub action_id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_required: Option<bool>,
}

pub fn active_control(load: &ControlLoad) -> Option<&AutoPlayControl> {
    match load {
        ControlLoad::Updated(control) | ControlLoad::MissingDefault(control) => Some(control),
        ControlLoad::Stale | ControlLoad::Malformed(_) => None,
    }
}

pub fn available_action_candidates(
    control: &AutoPlayControl,
    session: &AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
) -> Vec<ActionCandidate> {
    if control.mode != AutoPlayMode::Auto
        || control.require_confirmation
        || !command_state.ready_for_command
    {
        tracing::info!(
            "autoplay no candidates: mode={:?} require_confirmation={} ready={}",
            control.mode,
            control.require_confirmation,
            command_state.ready_for_command,
        );
        return vec![];
    }

    match state.screen_type.as_ref().map(|st| st.as_str()) {
        Some("COMBAT_REWARD") if control.allow_combat_rewards => {
            combat_reward_candidates(control, session, command_state, state)
        }
        Some("CARD_REWARD") if control.allow_card_rewards => {
            card_reward_candidates(command_state, state)
        }
        Some("BOSS_REWARD") if control.allow_boss_rewards => {
            boss_reward_candidates(command_state, state)
        }
        Some("REST") if control.allow_rest => rest_candidates(command_state, state),
        Some("EVENT") if control.allow_events => event_candidates(command_state, state),
        Some("SHOP_ROOM" | "SHOP_SCREEN") if control.allow_shop => {
            shop_candidates(session, command_state, state.floor)
        }
        Some("MAP") if control.allow_map => map_candidates(command_state),
        Some("NONE") if control.allow_combat => combat_candidates(command_state, state),
        Some("GRID") if control.allow_selection_screens => grid_candidates(command_state),
        Some("HAND_SELECT") if control.allow_selection_screens => {
            hand_select_candidates(command_state)
        }
        Some("CHEST") if control.allow_selection_screens => chest_candidates(command_state),
        Some("COMPLETE") if control.allow_selection_screens => {
            vec![ActionCandidate {
                kind: "proceed".to_string(),
                action_id: "complete:proceed".to_string(),
                label: "Proceed".to_string(),
                target_required: None,
            }]
        }
        _ => {
            tracing::info!(
                "autoplay no candidates: screen_type={:?}",
                state.screen_type,
            );
            vec![]
        }
    }
}

pub fn resolve_requested_action(
    control: &AutoPlayControl,
    session: &AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if !available_action_candidates(control, session, command_state, state)
        .iter()
        .any(|candidate| candidate.action_id == request.action_id && candidate.kind == request.kind)
    {
        return None;
    }

    match state.screen_type.as_ref().map(|st| st.as_str()) {
        Some("COMBAT_REWARD") => resolve_requested_combat_reward(command_state, request),
        Some("CARD_REWARD") => resolve_requested_card_reward(command_state, state, request),
        Some("BOSS_REWARD") => resolve_requested_boss_reward(state, request),
        Some("REST") => resolve_requested_rest(state, request),
        Some("EVENT") => resolve_requested_indexed("event:", state.event_choices.len(), request),
        Some("SHOP_ROOM" | "SHOP_SCREEN") => resolve_requested_shop(command_state, request),
        Some("MAP") => {
            resolve_requested_indexed("map:choice:", command_state.choice_list.len(), request)
        }
        Some("NONE") => resolve_requested_combat(state, request),
        Some("GRID") => {
            if request.action_id == "grid:confirm" && request.kind == "proceed" {
                Some(AutoPlayAction::Proceed)
            } else {
                resolve_requested_indexed("grid:", command_state.choice_list.len(), request)
            }
        }
        Some("HAND_SELECT") => {
            if request.action_id == "hand_select:confirm" && request.kind == "proceed" {
                Some(AutoPlayAction::Proceed)
            } else {
                resolve_requested_indexed("hand_select:", command_state.choice_list.len(), request)
            }
        }
        Some("CHEST") => {
            if request.action_id == "chest:proceed" && request.kind == "proceed" {
                Some(AutoPlayAction::Proceed)
            } else {
                resolve_requested_indexed("chest:", command_state.choice_list.len(), request)
            }
        }
        Some("COMPLETE") => {
            if request.action_id == "complete:proceed" && request.kind == "proceed" {
                Some(AutoPlayAction::Proceed)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn execute_action_to(writer: &mut impl Write, action: &AutoPlayAction) {
    match action {
        AutoPlayAction::Choose(index) => protocol::send_choose_to(writer, *index),
        AutoPlayAction::Play {
            hand_index,
            target_index,
        } => protocol::send_play_to(writer, *hand_index, *target_index),
        AutoPlayAction::Drink {
            slot_index,
            target_index,
        } => protocol::send_potion_to(writer, "use", *slot_index, *target_index),
        AutoPlayAction::End => protocol::send_end_to(writer),
        AutoPlayAction::Skip => protocol::send_skip_to(writer),
        AutoPlayAction::Proceed => protocol::send_proceed_to(writer),
        AutoPlayAction::Leave => protocol::send_leave_to(writer),
    }
}

fn candidate(kind: &str, action_id: String, label: String) -> ActionCandidate {
    ActionCandidate {
        kind: kind.to_string(),
        action_id,
        label,
        target_required: None,
    }
}

fn targeted_candidate(kind: &str, action_id: String, label: String) -> ActionCandidate {
    ActionCandidate {
        kind: kind.to_string(),
        action_id,
        label,
        target_required: Some(true),
    }
}

fn parse_index(action_id: &str, prefix: &str) -> Option<usize> {
    action_id.strip_prefix(prefix)?.parse().ok()
}

#[cfg(test)]
#[path = "tests/action_tests/mod.rs"]
mod tests;
