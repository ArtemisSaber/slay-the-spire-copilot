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
        return vec![];
    }

    match state.screen_type.as_deref() {
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
        _ => vec![],
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

    match state.screen_type.as_deref() {
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
        return vec![];
    }

    state
        .event_choices
        .iter()
        .enumerate()
        .map(|(index, choice)| candidate("choose", format!("event:{index}"), choice.clone()))
        .collect()
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
mod tests {
    use super::*;
    use crate::autoplay::control::{AutoPlayControl, ControlLoad};
    use crate::locales::Locale;
    use serde_json::{Value, json};

    fn state(raw: Value) -> NormalizedState {
        NormalizedState::from_raw(&raw, &Locale::load("en"))
    }

    fn command_state(raw: &Value) -> CommandState {
        CommandState::from_raw(raw)
    }

    fn request(kind: &str, action_id: &str) -> ActionRequest {
        ActionRequest {
            kind: kind.to_string(),
            action_id: action_id.to_string(),
            target_index: None,
        }
    }

    fn targeted_request(kind: &str, action_id: &str, target_index: usize) -> ActionRequest {
        ActionRequest {
            kind: kind.to_string(),
            action_id: action_id.to_string(),
            target_index: Some(target_index),
        }
    }

    fn resolve(
        raw: &Value,
        control: &AutoPlayControl,
        request: &ActionRequest,
    ) -> Option<AutoPlayAction> {
        resolve_requested_action(
            control,
            &AutoPlaySession::default(),
            &command_state(raw),
            &state(raw.clone()),
            request,
        )
    }

    #[test]
    fn missing_default_control_is_active_for_testing() {
        let load = ControlLoad::MissingDefault(AutoPlayControl::default_enabled());

        assert_eq!(
            active_control(&load),
            Some(&AutoPlayControl::default_enabled())
        );
    }

    #[test]
    fn malformed_control_is_not_active() {
        let load = ControlLoad::Malformed("bad json".to_string());

        assert_eq!(active_control(&load), None);
    }

    #[test]
    fn paused_control_has_no_action_candidates() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["gold"]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.mode = crate::autoplay::control::AutoPlayMode::Paused;

        assert!(
            available_action_candidates(
                &control,
                &AutoPlaySession::default(),
                &command_state(&raw),
                &state(raw)
            )
            .is_empty()
        );
    }

    #[test]
    fn not_ready_has_no_action_candidates() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": false,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["gold"]
            }
        });

        assert!(
            available_action_candidates(
                &AutoPlayControl::default_enabled(),
                &AutoPlaySession::default(),
                &command_state(&raw),
                &state(raw)
            )
            .is_empty()
        );
    }

    #[test]
    fn combat_reward_request_collects_requested_reward() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["gold", "card"]
            }
        });

        assert_eq!(
            resolve(
                &raw,
                &AutoPlayControl::default_enabled(),
                &request("choose", "combat_reward:card:1")
            ),
            Some(AutoPlayAction::Choose(1))
        );
    }

    #[test]
    fn combat_reward_request_can_proceed_when_empty() {
        let raw = json!({
            "available_commands": ["proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": []
            }
        });

        assert_eq!(
            resolve(
                &raw,
                &AutoPlayControl::default_enabled(),
                &request("proceed", "combat_reward:proceed")
            ),
            Some(AutoPlayAction::Proceed)
        );
    }

    #[test]
    fn card_reward_request_can_choose_or_skip() {
        let raw = json!({
            "available_commands": ["choose", "skip"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "CARD_REWARD",
                "screen_state": {
                    "skip_available": true,
                    "cards": [{"id": "Anger", "name": "Anger"}]
                }
            }
        });

        assert_eq!(
            resolve(
                &raw,
                &AutoPlayControl::default_enabled(),
                &request("choose", "card_reward:0")
            ),
            Some(AutoPlayAction::Choose(0))
        );
        assert_eq!(
            resolve(
                &raw,
                &AutoPlayControl::default_enabled(),
                &request("skip", "card_reward:skip")
            ),
            Some(AutoPlayAction::Skip)
        );
    }

    #[test]
    fn boss_reward_request_chooses_requested_relic() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "BOSS_REWARD",
                "screen_state": {
                    "relics": [
                        {"id": "Coffee Dripper", "name": "Coffee Dripper"},
                        {"id": "Astrolabe", "name": "Astrolabe"}
                    ]
                }
            }
        });

        assert_eq!(
            resolve(
                &raw,
                &AutoPlayControl::default_enabled(),
                &request("choose", "boss_relic:1")
            ),
            Some(AutoPlayAction::Choose(1))
        );
    }

    #[test]
    fn rest_request_maps_option_to_index() {
        let raw = json!({
            "available_commands": ["choose", "return"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "REST",
                "screen_state": {
                    "rest_options": ["rest", "smith", "toke"]
                }
            }
        });

        assert_eq!(
            resolve(
                &raw,
                &AutoPlayControl::default_enabled(),
                &request("choose", "rest:smith")
            ),
            Some(AutoPlayAction::Choose(1))
        );
    }

    #[test]
    fn event_request_maps_requested_index() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["Fight", "Leave"]
            }
        });

        assert_eq!(
            resolve(
                &raw,
                &AutoPlayControl::default_enabled(),
                &request("choose", "event:1")
            ),
            Some(AutoPlayAction::Choose(1))
        );
    }

    #[test]
    fn shop_request_can_leave_or_choose_index() {
        let raw = json!({
            "available_commands": ["choose", "leave"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "SHOP_SCREEN",
                "choice_list": ["purge", "Strike"]
            }
        });
        let control = AutoPlayControl::default_enabled();

        assert_eq!(
            resolve(&raw, &control, &request("leave", "shop:leave")),
            Some(AutoPlayAction::Leave)
        );
        assert_eq!(
            resolve(&raw, &control, &request("choose", "shop:choice:1")),
            Some(AutoPlayAction::Choose(1))
        );
    }

    #[test]
    fn map_request_maps_requested_index() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "MAP",
                "choice_list": ["M", "?"]
            }
        });
        let control = AutoPlayControl::default_enabled();

        assert_eq!(
            resolve(&raw, &control, &request("choose", "map:choice:1")),
            Some(AutoPlayAction::Choose(1))
        );
    }

    #[test]
    fn combat_request_plays_requested_card_and_target() {
        let raw = json!({
            "available_commands": ["play", "end"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "player": {"energy": 3, "block": 0, "powers": []},
                    "hand": [
                        {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK", "uuid": "strike-1", "has_target": true},
                        {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "defend-1", "has_target": false}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_combat = true;

        assert_eq!(
            resolve(
                &raw,
                &control,
                &targeted_request("play", "combat:play:strike-1", 0)
            ),
            Some(AutoPlayAction::Play {
                hand_index: 0,
                target_index: Some(0),
            })
        );
    }

    #[test]
    fn combat_rejects_unaffordable_requested_card() {
        let raw = json!({
            "available_commands": ["play", "end"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "player": {"energy": 1, "block": 0, "powers": []},
                    "hand": [
                        {"id": "Bash", "name": "Bash", "cost": 2, "type": "ATTACK", "uuid": "bash-1", "has_target": true}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_combat = true;

        assert_eq!(
            resolve(
                &raw,
                &control,
                &targeted_request("play", "combat:play:bash-1", 0)
            ),
            None
        );
    }

    #[test]
    fn invalid_requested_action_is_rejected() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["Leave"]
            }
        });

        assert_eq!(
            resolve(
                &raw,
                &AutoPlayControl::default_enabled(),
                &request("choose", "event:9")
            ),
            None
        );
    }

    #[test]
    fn candidates_include_all_current_event_choices() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["Fight", "Leave"]
            }
        });
        let candidates = available_action_candidates(
            &AutoPlayControl::default_enabled(),
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
        );

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.action_id.as_str())
                .collect::<Vec<_>>(),
            vec!["event:0", "event:1"]
        );
    }

    #[test]
    fn combat_candidates_require_targets_for_attacks() {
        let raw = json!({
            "available_commands": ["play", "end"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "player": {"energy": 1, "block": 0, "powers": []},
                    "hand": [
                        {"id": "Bash", "name": "Bash", "cost": 2, "type": "ATTACK", "uuid": "bash-1", "has_target": true},
                        {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "defend-1", "has_target": false}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_combat = true;
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
        );

        assert!(candidates.iter().any(|candidate| {
            candidate.action_id == "combat:play:bash-1" && candidate.target_required == Some(true)
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.action_id == "combat:play:defend-1" && candidate.target_required.is_none()
        }));
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.action_id == "combat:end")
        );
    }

    #[test]
    fn execute_action_writes_protocol_command() {
        let mut buf = Vec::new();

        execute_action_to(&mut buf, &AutoPlayAction::Choose(2));
        execute_action_to(
            &mut buf,
            &AutoPlayAction::Play {
                hand_index: 1,
                target_index: Some(0),
            },
        );
        execute_action_to(
            &mut buf,
            &AutoPlayAction::Drink {
                slot_index: 0,
                target_index: None,
            },
        );
        execute_action_to(
            &mut buf,
            &AutoPlayAction::Drink {
                slot_index: 1,
                target_index: Some(2),
            },
        );
        execute_action_to(&mut buf, &AutoPlayAction::End);
        execute_action_to(&mut buf, &AutoPlayAction::Skip);
        execute_action_to(&mut buf, &AutoPlayAction::Proceed);
        execute_action_to(&mut buf, &AutoPlayAction::Leave);

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(
            lines,
            vec![
                "choose 2",
                "play 2 0",
                "potion use 0",
                "potion use 1 2",
                "end",
                "skip",
                "proceed",
                "leave"
            ]
        );
    }

    #[test]
    fn combat_skill_with_has_target_needs_target() {
        let raw = json!({
            "available_commands": ["play", "end"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "player": {"energy": 3, "block": 0, "powers": []},
                    "hand": [
                        {"id": "Neutralize", "name": "Neutralize", "cost": 0, "type": "SKILL", "uuid": "neut-1", "has_target": true, "is_playable": true}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_combat = true;
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
        );
        let neut = candidates
            .iter()
            .find(|c| c.action_id == "combat:play:neut-1")
            .unwrap();
        assert_eq!(
            neut.target_required,
            Some(true),
            "has_target skill should require target"
        );
    }

    #[test]
    fn combat_skill_without_has_target_does_not_need_target() {
        let raw = json!({
            "available_commands": ["play", "end"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "player": {"energy": 3, "block": 0, "powers": []},
                    "hand": [
                        {"id": "Defend", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "def-1", "has_target": false, "is_playable": true}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_combat = true;
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
        );
        let defend = candidates
            .iter()
            .find(|c| c.action_id == "combat:play:def-1")
            .unwrap();
        assert_eq!(
            defend.target_required, None,
            "defend has no target, should not require one"
        );
    }

    #[test]
    fn hand_select_candidates_use_choice_list() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "HAND_SELECT",
                "choice_list": ["Strike", "Defend"]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_selection_screens = true;
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
        );
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].action_id, "hand_select:0");
        assert_eq!(candidates[1].action_id, "hand_select:1");

        let raw_with_confirm = json!({
            "available_commands": ["choose", "confirm"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "HAND_SELECT",
                "choice_list": ["Strike"]
            }
        });
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw_with_confirm),
            &state(raw_with_confirm),
        );
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().any(|c| c.action_id == "hand_select:0"));
        assert!(
            candidates
                .iter()
                .any(|c| c.action_id == "hand_select:confirm" && c.kind == "proceed")
        );
    }

    #[test]
    fn grid_candidates_use_choice_list() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "GRID",
                "choice_list": ["Strike", "Strike", "Defend"]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_selection_screens = true;
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
        );
        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].action_id, "grid:0");
        assert_eq!(candidates[2].action_id, "grid:2");

        let raw_with_confirm = json!({
            "available_commands": ["choose", "confirm"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "GRID",
                "choice_list": ["Strike"]
            }
        });
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw_with_confirm),
            &state(raw_with_confirm),
        );
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().any(|c| c.action_id == "grid:0"));
        assert!(
            candidates
                .iter()
                .any(|c| c.action_id == "grid:confirm" && c.kind == "proceed")
        );
    }

    #[test]
    fn rest_candidates_includes_proceed_when_available() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "REST",
                "screen_state": {
                    "rest_options": ["rest", "smith"]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_rest = true;
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state(&raw),
            &state(raw),
        );
        assert_eq!(candidates.len(), 3);
        assert!(candidates.iter().any(|c| c.action_id == "rest:rest"));
        assert!(candidates.iter().any(|c| c.action_id == "rest:smith"));
        assert!(
            candidates
                .iter()
                .any(|c| c.action_id == "rest:proceed" && c.kind == "proceed")
        );
    }

    #[test]
    fn resolve_rest_proceed_returns_proceed() {
        let raw = json!({
            "game_state": {
                "screen_type": "REST",
                "screen_state": {
                    "rest_options": []
                }
            }
        });
        let state = state(raw);
        let request = ActionRequest {
            kind: "proceed".to_string(),
            action_id: "rest:proceed".to_string(),
            target_index: None,
        };
        assert_eq!(
            resolve_requested_rest(&state, &request),
            Some(AutoPlayAction::Proceed)
        );
    }

    #[test]
    fn resolve_grid_confirm_returns_proceed() {
        let raw = json!({
            "available_commands": ["confirm"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "GRID",
                "choice_list": []
            }
        });
        let command_state = command_state(&raw);
        let state = state(raw);
        let mut control = AutoPlayControl::default_enabled();
        control.allow_selection_screens = true;
        let request = ActionRequest {
            kind: "proceed".to_string(),
            action_id: "grid:confirm".to_string(),
            target_index: None,
        };
        assert_eq!(
            resolve_requested_action(
                &control,
                &AutoPlaySession::default(),
                &command_state,
                &state,
                &request
            ),
            Some(AutoPlayAction::Proceed)
        );
    }

    #[test]
    fn resolve_hand_select_confirm_returns_proceed() {
        let raw = json!({
            "available_commands": ["confirm"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "HAND_SELECT",
                "choice_list": []
            }
        });
        let command_state = command_state(&raw);
        let state = state(raw);
        let mut control = AutoPlayControl::default_enabled();
        control.allow_selection_screens = true;
        let request = ActionRequest {
            kind: "proceed".to_string(),
            action_id: "hand_select:confirm".to_string(),
            target_index: None,
        };
        assert_eq!(
            resolve_requested_action(
                &control,
                &AutoPlaySession::default(),
                &command_state,
                &state,
                &request
            ),
            Some(AutoPlayAction::Proceed)
        );
    }

    #[test]
    fn complete_candidates_returns_proceed() {
        let raw = json!({
            "game_state": {
                "screen_type": "COMPLETE",
                "screen_state": {},
                "seed": -582230291998696632_i64,
                "relics": [],
                "deck": [],
                "map": [],
                "floor": 50,
            },
            "available_commands": ["proceed", "wait", "state"],
            "ready_for_command": true,
            "in_game": true,
        });
        let command_state = command_state(&raw);
        let mut control = AutoPlayControl::default_enabled();
        control.allow_selection_screens = true;
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state(raw),
        );
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, "proceed");
        assert_eq!(candidates[0].action_id, "complete:proceed");
    }

    #[test]
    fn resolve_complete_proceed_returns_proceed() {
        let raw = json!({
            "game_state": {
                "screen_type": "COMPLETE",
                "screen_state": {},
                "seed": -582230291998696632_i64,
                "relics": [],
                "deck": [],
                "map": [],
                "floor": 50,
            },
            "available_commands": ["proceed", "wait", "state"],
            "ready_for_command": true,
            "in_game": true,
        });
        let command_state = command_state(&raw);
        let state = state(raw);
        let mut control = AutoPlayControl::default_enabled();
        control.allow_selection_screens = true;
        let request = ActionRequest {
            kind: "proceed".to_string(),
            action_id: "complete:proceed".to_string(),
            target_index: None,
        };
        assert_eq!(
            resolve_requested_action(
                &control,
                &AutoPlaySession::default(),
                &command_state,
                &state,
                &request
            ),
            Some(AutoPlayAction::Proceed)
        );
    }

    #[test]
    fn potion_action_id_uses_raw_slot_with_gaps() {
        use crate::state::PotionInfo;
        let state = NormalizedState {
            screen_type: Some("NONE".into()),
            potions: vec![
                PotionInfo {
                    slot: 1,
                    name: "能量药水".into(),
                    description: "获得 2 点能量。".into(),
                    price: None,
                    can_use: true,
                    can_discard: true,
                    requires_target: false,
                },
                PotionInfo {
                    slot: 2,
                    name: "格挡药水".into(),
                    description: "获得 12 点 格挡 。".into(),
                    price: None,
                    can_use: true,
                    can_discard: true,
                    requires_target: false,
                },
            ],
            empty_potion_slots: 1,
            monsters: vec![crate::state::MonsterInfo {
                name: "Test".into(),
                index: 0,
                current_hp: Some(10),
                max_hp: Some(10),
                block: Some(0),
                intent: Some("ATTACK".into()),
                damage: Some(5),
                hits: Some(1),
                monster_powers: vec![],
                can_be_killed: false,
                is_scaling: false,
                monster_id: None,
            }],
            energy: Some(0),
            ..NormalizedState::default()
        };
        let raw = serde_json::json!({
            "available_commands": ["end", "potion"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "hand": [],
                    "monsters": [{"name": "Test", "current_hp": 10, "max_hp": 10, "block": 0, "intent": "ATTACK", "move_adjusted_damage": 5, "move_hits": 1, "powers": [], "is_gone": false, "half_dead": false}],
                    "player": {"energy": 0, "block": 0, "powers": []},
                    "turn": 1
                },
                "potions": [
                    {"id": "Potion Slot", "can_use": false},
                    {"id": "Energy Potion", "can_use": true, "name": "能量药水"},
                    {"id": "Block Potion", "can_use": true, "name": "格挡药水"}
                ]
            }
        });
        let command_state = command_state(&raw);
        let candidates = available_action_candidates(
            &AutoPlayControl::default_enabled(),
            &AutoPlaySession::default(),
            &command_state,
            &state,
        );
        let potion_candidates: Vec<_> = candidates.iter().filter(|c| c.kind == "drink").collect();
        assert_eq!(potion_candidates.len(), 2);
        assert_eq!(
            potion_candidates[0].action_id, "combat:potion:1",
            "energy potion should use raw slot 1"
        );
        assert_eq!(
            potion_candidates[1].action_id, "combat:potion:2",
            "block potion should use raw slot 2"
        );
    }
}
