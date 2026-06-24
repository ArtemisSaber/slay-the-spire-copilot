use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlayMode, ControlLoad};
use crate::protocol;
use crate::state::NormalizedState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoPlayAction {
    Choose(usize),
    Play {
        hand_index: usize,
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
            combat_reward_candidates(control, command_state)
        }
        Some("CARD_REWARD") if control.allow_card_rewards => {
            card_reward_candidates(command_state, state)
        }
        Some("BOSS_REWARD") if control.allow_boss_rewards => {
            boss_reward_candidates(command_state, state)
        }
        Some("REST") if control.allow_rest => rest_candidates(command_state, state),
        Some("EVENT") if control.allow_events => event_candidates(command_state, state),
        Some("SHOP_SCREEN") if control.allow_shop => shop_candidates(command_state),
        Some("MAP") if control.allow_map => map_candidates(command_state),
        Some("NONE") if control.allow_combat => combat_candidates(command_state, state),
        Some("GRID") if control.allow_selection_screens => {
            grid_candidates(command_state)
        }
        _ => vec![],
    }
}

pub fn resolve_requested_action(
    control: &AutoPlayControl,
    command_state: &CommandState,
    state: &NormalizedState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
    if !available_action_candidates(control, command_state, state)
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
        Some("SHOP_SCREEN") => resolve_requested_shop(command_state, request),
        Some("MAP") => {
            resolve_requested_indexed("map:choice:", command_state.choice_list.len(), request)
        }
        Some("NONE") => resolve_requested_combat(state, request),
        Some("GRID") => {
            resolve_requested_indexed("grid:", command_state.choice_list.len(), request)
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
    command_state: &CommandState,
) -> Vec<ActionCandidate> {
    let mut candidates = vec![];

    if command_state.has_command("choose") {
        for (index, choice) in command_state.choice_list.iter().enumerate() {
            let allowed = matches!(
                choice.as_str(),
                "gold" | "relic" | "potion" | "emerald_key" | "sapphire_key"
            ) || (choice == "card" && control.allow_card_rewards);
            if allowed {
                candidates.push(candidate(
                    "choose",
                    format!("combat_reward:{choice}:{index}"),
                    format!("Collect {choice}"),
                ));
            }
        }
    }

    if command_state.has_command("proceed") && command_state.choice_list.is_empty() {
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
    if !command_state.has_command("choose") {
        return vec![];
    }

    state
        .rest_options
        .iter()
        .map(|option| candidate("choose", format!("rest:{option}"), option.clone()))
        .collect()
}

fn resolve_requested_rest(
    state: &NormalizedState,
    request: &ActionRequest,
) -> Option<AutoPlayAction> {
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

fn shop_candidates(command_state: &CommandState) -> Vec<ActionCandidate> {
    let mut candidates = vec![];

    if command_state.has_command("choose") {
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

fn grid_candidates(command_state: &CommandState) -> Vec<ActionCandidate> {
    if !command_state.has_command("choose") {
        return vec![];
    }

    command_state
        .choice_list
        .iter()
        .enumerate()
        .map(|(index, choice)| candidate("choose", format!("grid:{index}"), choice.clone()))
        .collect()
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
            if card.card_type == "ATTACK" {
                candidates.push(targeted_candidate("play", action_id, label));
            } else {
                candidates.push(candidate("play", action_id, label));
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

    if request.kind != "play" {
        return None;
    }

    let uuid = request.action_id.strip_prefix("combat:play:")?;
    let (hand_index, card) = state
        .hand
        .iter()
        .enumerate()
        .find(|(_, card)| card.uuid.as_deref() == Some(uuid))?;

    if card.cost > state.energy.unwrap_or(0) {
        return None;
    }

    let target_index = if card.card_type == "ATTACK" {
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

    Some(AutoPlayAction::Play {
        hand_index,
        target_index,
    })
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
        resolve_requested_action(control, &command_state(raw), &state(raw.clone()), request)
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
            available_action_candidates(&control, &command_state(&raw), &state(raw)).is_empty()
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
                        {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK", "uuid": "strike-1"},
                        {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "defend-1"}
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
                        {"id": "Bash", "name": "Bash", "cost": 2, "type": "ATTACK", "uuid": "bash-1"}
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
                        {"id": "Bash", "name": "Bash", "cost": 2, "type": "ATTACK", "uuid": "bash-1"},
                        {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "defend-1"}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_combat = true;
        let candidates = available_action_candidates(&control, &command_state(&raw), &state(raw));

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
        execute_action_to(&mut buf, &AutoPlayAction::End);
        execute_action_to(&mut buf, &AutoPlayAction::Skip);
        execute_action_to(&mut buf, &AutoPlayAction::Proceed);
        execute_action_to(&mut buf, &AutoPlayAction::Leave);

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(
            lines,
            vec!["choose 2", "play 1 0", "end", "skip", "proceed", "leave"]
        );
    }
}
