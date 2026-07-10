use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlayMode, AutoPlaySession, ControlLoad};
use crate::protocol;
use crate::state::NormalizedState;

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

fn combat_reward_candidates(
    control: &AutoPlayControl,
    session: &AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
) -> Vec<ActionCandidate> {
    let mut candidates = vec![];

    let skip_potion = session.skipped_combat_reward_potion && state.empty_potion_slots == 0;
    let skip_card = session.skipped_combat_reward_card;

    if command_state.has_command("choose") {
        for (index, choice) in command_state.choice_list.iter().enumerate() {
            let allowed = matches!(
                choice.as_str(),
                "gold" | "relic" | "stolen_gold" | "potion" | "emerald_key" | "sapphire_key"
            ) || (choice == "card" && control.allow_card_rewards);
            let skip_on_full = choice == "potion" && skip_potion;
            let skip_on_skipped = choice == "card" && skip_card;
            if allowed && !skip_on_full && !skip_on_skipped {
                candidates.push(candidate(
                    "choose",
                    format!("combat_reward:{choice}:{index}"),
                    format!("Collect {choice}"),
                ));
            }
        }
    }

    let visible = !candidates.is_empty();
    if command_state.has_command("proceed") && !visible {
        candidates.push(candidate(
            "proceed",
            "combat_reward:proceed".to_string(),
            "Proceed".to_string(),
        ));
    }

    candidates
}

fn resolve_requested_combat_reward(
    command_state: &CommandState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.action_id == "combat_reward:proceed" && request.kind == "proceed" {
        return Some(AutoPlayAction::Proceed);
    }

    if request.kind != "choose" || !command_state.has_command("choose") {
        return None;
    }

    let index = request.action_id.rsplit(':').next()?.parse().ok()?;
    Some(AutoPlayAction::Choose(index))
}

fn card_reward_candidates(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Vec<ActionCandidate> {
    let mut candidates = vec![];

    if command_state.has_command("choose") {
        let prefix = if state.is_boss_card_reward() {
            "boss_card_reward"
        } else {
            "card_reward"
        };
        for (index, card) in state.card_reward_choices.iter().enumerate() {
            candidates.push(candidate(
                "choose",
                format!("{prefix}:{index}"),
                card.name.clone(),
            ));
        }
    }

    if state.skip_available && command_state.has_command("skip") {
        let id = if state.is_boss_card_reward() {
            "boss_card_reward:skip"
        } else {
            "card_reward:skip"
        };
        candidates.push(candidate("skip", id.to_string(), "Skip".to_string()));
    }

    candidates
}

fn resolve_requested_card_reward(
    command_state: &CommandState,
    state: &NormalizedState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    let prefix = if state.is_boss_card_reward() {
        "boss_card_reward:"
    } else {
        "card_reward:"
    };

    if request.action_id == format!("{prefix}skip") && request.kind == "skip" {
        return Some(AutoPlayAction::Skip);
    }

    if request.kind != "choose" || !command_state.has_command("choose") {
        return None;
    }

    let index = parse_index(&request.action_id, prefix)?;
    (index < state.card_reward_choices.len()).then_some(AutoPlayAction::Choose(index))
}

fn boss_reward_candidates(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Vec<ActionCandidate> {
    if !command_state.has_command("choose") {
        return vec![];
    }

    state
        .boss_relic_choices
        .iter()
        .enumerate()
        .map(|(index, relic)| {
            candidate("choose", format!("boss_relic:{index}"), relic.name.clone())
        })
        .collect()
}

fn resolve_requested_boss_reward(
    state: &NormalizedState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.kind != "choose" {
        return None;
    }

    let index = parse_index(&request.action_id, "boss_relic:")?;
    (index < state.boss_relic_choices.len()).then_some(AutoPlayAction::Choose(index))
}

fn rest_candidates(command_state: &CommandState, state: &NormalizedState) -> Vec<ActionCandidate> {
    let mut candidates: Vec<ActionCandidate> = vec![];

    if command_state.has_command("choose") {
        candidates.extend(
            state
                .rest_options
                .iter()
                .map(|option| candidate("choose", format!("rest:{option}"), option.clone())),
        );
    }

    if command_state.has_command("proceed") {
        candidates.push(candidate(
            "proceed",
            "rest:proceed".to_string(),
            "Proceed".to_string(),
        ));
    }

    candidates
}

fn resolve_requested_rest(
    state: &NormalizedState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.action_id == "rest:proceed" && request.kind == "proceed" {
        return Some(AutoPlayAction::Proceed);
    }

    if request.kind != "choose" {
        return None;
    }

    let option = request.action_id.strip_prefix("rest:")?;
    state
        .rest_options
        .iter()
        .position(|candidate| candidate == option)
        .map(AutoPlayAction::Choose)
}

fn event_candidates(command_state: &CommandState, state: &NormalizedState) -> Vec<ActionCandidate> {
    if !command_state.has_command("choose") {
        tracing::info!(
            "event_candidates empty: no 'choose' command commands={:?}",
            command_state.available_commands,
        );
        return vec![];
    }

    let candidates: Vec<_> = state
        .event_choices
        .iter()
        .enumerate()
        .map(|(index, choice)| candidate("choose", format!("event:{index}"), choice.clone()))
        .collect();
    if candidates.is_empty() {
        tracing::info!("event_candidates empty: event_choices is empty");
    }
    candidates
}

fn resolve_requested_indexed(
    prefix: &str,
    len: usize,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.kind != "choose" {
        return None;
    }

    let index = parse_index(&request.action_id, prefix)?;
    (index < len).then_some(AutoPlayAction::Choose(index))
}

fn shop_candidates(
    session: &AutoPlaySession,
    command_state: &CommandState,
    floor: Option<i64>,
) -> Vec<ActionCandidate> {
    let mut candidates = vec![];

    let already_entered = match (session.last_shop_room_floor, floor) {
        (Some(last), Some(current)) => last == current,
        _ => false,
    };

    if already_entered {
        if command_state.has_command("proceed") {
            candidates.push(candidate(
                "proceed",
                "shop:proceed".to_string(),
                "Proceed".to_string(),
            ));
        }
    } else if command_state.has_command("choose") {
        for (index, choice) in command_state.choice_list.iter().enumerate() {
            candidates.push(candidate(
                "choose",
                format!("shop:choice:{index}"),
                choice.clone(),
            ));
        }
    }

    if command_state.has_command("leave") {
        candidates.push(candidate(
            "leave",
            "shop:leave".to_string(),
            "Leave shop".to_string(),
        ));
    }

    candidates
}

fn resolve_requested_shop(
    command_state: &CommandState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if request.action_id == "shop:leave" && request.kind == "leave" {
        return Some(AutoPlayAction::Leave);
    }

    if request.action_id == "shop:proceed" && request.kind == "proceed" {
        return Some(AutoPlayAction::Proceed);
    }

    resolve_requested_indexed("shop:choice:", command_state.choice_list.len(), request)
}

fn map_candidates(command_state: &CommandState) -> Vec<ActionCandidate> {
    if !command_state.has_command("choose") {
        return vec![];
    }

    command_state
        .choice_list
        .iter()
        .enumerate()
        .map(|(index, choice)| candidate("choose", format!("map:choice:{index}"), choice.clone()))
        .collect()
}

fn chest_candidates(command_state: &CommandState) -> Vec<ActionCandidate> {
    if command_state.has_command("choose") {
        return command_state
            .choice_list
            .iter()
            .enumerate()
            .map(|(index, choice)| candidate("choose", format!("chest:{index}"), choice.clone()))
            .collect();
    }

    if command_state.has_command("proceed") {
        return vec![ActionCandidate {
            kind: "proceed".to_string(),
            action_id: "chest:proceed".to_string(),
            label: "Proceed".to_string(),
            target_required: None,
        }];
    }

    vec![]
}

fn grid_candidates(command_state: &CommandState) -> Vec<ActionCandidate> {
    let mut candidates: Vec<ActionCandidate> = vec![];

    if command_state.has_command("choose") {
        candidates.extend(
            command_state
                .choice_list
                .iter()
                .enumerate()
                .map(|(index, choice)| {
                    candidate("choose", format!("grid:{index}"), choice.clone())
                }),
        );
    }

    if command_state.has_command("confirm") {
        candidates.push(candidate(
            "proceed",
            "grid:confirm".to_string(),
            "Confirm".to_string(),
        ));
    }

    candidates
}

fn hand_select_candidates(command_state: &CommandState) -> Vec<ActionCandidate> {
    let mut candidates: Vec<ActionCandidate> = vec![];

    if command_state.has_command("choose") {
        candidates.extend(
            command_state
                .choice_list
                .iter()
                .enumerate()
                .map(|(index, choice)| {
                    candidate("choose", format!("hand_select:{index}"), choice.clone())
                }),
        );
    }

    if command_state.has_command("confirm") {
        candidates.push(candidate(
            "proceed",
            "hand_select:confirm".to_string(),
            "Confirm".to_string(),
        ));
    }

    candidates
}

fn combat_candidates(
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
        for potion in state.potions.iter() {
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

fn resolve_requested_combat(
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

#[cfg(test)]
#[path = "tests/action_tests/mod.rs"]
mod tests;
