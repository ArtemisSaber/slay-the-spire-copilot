use anyhow::{Context, bail};
use serde::Deserialize;
use serde_json::json;

use crate::autoplay::action::{
    ActionCandidate, ActionRequest, AutoPlayAction, available_action_candidates,
    resolve_requested_action,
};
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
use crate::llm::{Effort, LlmProvider};
use crate::locales::Locale;
use crate::prompt;
use crate::state::NormalizedState;

const MAX_LLM_ATTEMPTS: usize = 3;

#[derive(Debug, Deserialize)]
struct PlannerResponse {
    schema_version: u32,
    actions: Vec<ActionRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RejectedAttempt {
    attempt: usize,
    rejected_action: Option<ActionRequestSummary>,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct ActionRequestSummary {
    kind: String,
    action_id: String,
    target_index: Option<usize>,
}

pub async fn plan_action(
    provider: &LlmProvider,
    control: &mut AutoPlayControl,
    session: &mut AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
) -> anyhow::Result<Option<AutoPlayAction>> {
    if state.screen_type.as_deref() != Some("COMBAT_REWARD") {
        session.skipped_combat_reward_potion = false;
    }

    let candidates = available_action_candidates(control, session, command_state, state);
    if candidates.is_empty() {
        return Ok(None);
    }

    if let Some(action) =
        try_deterministic_action(control, session, command_state, state, &candidates)
    {
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        return Ok(Some(action));
    }

    let effort = state
        .screen_type
        .as_deref()
        .map(|st| Effort::from_screen_type(st, !state.monsters.is_empty()))
        .unwrap_or(Effort::Medium);

    let mut rejections = vec![];
    for attempt in 1..=MAX_LLM_ATTEMPTS {
        let prompt = build_planner_prompt(
            command_state,
            state,
            locale,
            shop_visited,
            &candidates,
            &rejections,
        )?;
        match provider
            .query_autoplay_action(&prompt, effort, locale)
            .await
        {
            Ok(response) => {
                match parse_planner_response(&response, control, command_state, state, &candidates)
                {
                    Ok(action) => {
                        if let Some((potion_index, AutoPlayAction::Choose(chosen))) =
                            potion_in_full_slots_was_rejected(
                                &candidates,
                                command_state,
                                state,
                                &action,
                            )
                            && chosen != potion_index
                        {
                            session.skipped_combat_reward_potion = true;
                        }
                        return Ok(action);
                    }
                    Err(e) => rejections.push(RejectedAttempt {
                        attempt,
                        rejected_action: rejected_action_from_response(&response),
                        reason: e.to_string(),
                    }),
                }
            }
            Err(e) => rejections.push(RejectedAttempt {
                attempt,
                rejected_action: None,
                reason: format!("LLM query failed: {e}"),
            }),
        }
    }

    Ok(fallback_action(control, command_state, state))
}

fn potion_in_full_slots_was_rejected(
    candidates: &[ActionCandidate],
    command_state: &CommandState,
    state: &NormalizedState,
    action: &Option<AutoPlayAction>,
) -> Option<(usize, AutoPlayAction)> {
    if state.screen_type.as_deref() != Some("COMBAT_REWARD") {
        return None;
    }
    if state.empty_potion_slots > 0 {
        return None;
    }
    let potion_index = command_state
        .choice_list
        .iter()
        .position(|c| c == "potion")?;
    let potion_candidate_selected = candidates.iter().any(|c| {
        c.action_id == format!("combat_reward:potion:{potion_index}") && c.kind == "choose"
    });
    if !potion_candidate_selected {
        return None;
    }
    action.as_ref().map(|a| (potion_index, a.clone()))
}

fn try_deterministic_action(
    control: &mut AutoPlayControl,
    session: &mut AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
    candidates: &[ActionCandidate],
) -> Option<AutoPlayAction> {
    if candidates.len() == 1 {
        let sole = &candidates[0];
        let request = ActionRequest {
            kind: sole.kind.clone(),
            action_id: sole.action_id.clone(),
            target_index: sole.target_required.and(Some(0)),
        };
        let action = resolve_requested_action(control, session, command_state, state, &request)?;

        if state.screen_type.as_deref() == Some("SHOP_ROOM") && action == AutoPlayAction::Choose(0)
        {
            session.last_shop_room_floor = state.floor;
        }

        return Some(action);
    }

    // COMBAT_REWARD: deterministic when potion + empty slots, otherwise use priority fallback.
    if state.screen_type.as_deref() == Some("COMBAT_REWARD") {
        // Potion with empty slots — always pick, no LLM needed.
        if let Some(index) = command_state.choice_list.iter().position(|c| c == "potion")
            && state.empty_potion_slots > 0
            && command_state.has_command("choose")
        {
            return Some(AutoPlayAction::Choose(index));
        }
        // All other rewards — deterministic priority (gold → relic → potion → keys → card → proceed).
        // If potion is in the list but slots are full, fall through to LLM.
        if !command_state.choice_list.iter().any(|c| c == "potion") || state.empty_potion_slots > 0
        {
            return fallback_combat_reward_action(control, command_state);
        }
        // Potion with full slots — let LLM decide (discard/use or skip).
        return None;
    }

    None
}

fn build_planner_prompt(
    command_state: &CommandState,
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
    candidates: &[ActionCandidate],
    rejections: &[RejectedAttempt],
) -> anyhow::Result<String> {
    let localized_status_context = prompt::build_prompt(state, locale, shop_visited);
    let payload = json!({
        "task": "Choose exactly one action_id from available_actions. Return strict JSON only. Use localized_status_context as the strategy context.",
        "language": locale.language_name,
        "schema": {
            "schema_version": 1,
            "actions": [{
                "kind": "choose|skip|proceed|play|end|leave",
                "action_id": "one of available_actions.action_id",
                "target_index": "required only when target_required is true",
                "label": "short label",
                "reason": "short reason",
                "risk": "short risk or empty string"
            }]
        },
        "localized_status_context": localized_status_context,
        "state": state_summary(state),
        "available_commands": command_state.available_commands,
        "choice_list": command_state.choice_list,
        "available_actions": candidates,
        "rejected_attempts": rejections,
    });

    serde_json::to_string_pretty(&payload).context("failed to build autoplay planner prompt")
}

fn state_summary(state: &NormalizedState) -> serde_json::Value {
    json!({
        "screen_type": state.screen_type,
        "floor": state.floor,
        "hp": {
            "current": state.current_hp,
            "max": state.max_hp,
        },
        "gold": state.gold,
        "energy": state.energy,
        "incoming_damage": state.incoming_damage,
        "hand": state.hand.iter().enumerate().map(|(index, card)| {
            json!({
                "index": index,
                "id": card.id,
                "uuid": card.uuid,
                "name": card.name,
                "cost": card.cost,
                "type": card.card_type,
            })
        }).collect::<Vec<_>>(),
        "potions": state.potions.iter().enumerate().map(|(slot, potion)| {
            json!({
                "slot": slot,
                "name": potion.name,
                "description": potion.description,
                "can_use": potion.can_use,
                "requires_target": potion.requires_target,
            })
        }).collect::<Vec<_>>(),
        "monsters": state.monsters.iter().map(|monster| {
            json!({
                "index": monster.index,
                "name": monster.name,
                "hp": monster.current_hp,
                "block": monster.block,
                "intent": monster.intent,
                "damage": monster.damage,
            })
        }).collect::<Vec<_>>(),
        "card_reward_choices": state.card_reward_choices.iter().enumerate().map(|(index, card)| {
            json!({
                "index": index,
                "id": card.id,
                "name": card.name,
                "cost": card.cost,
                "type": card.card_type,
            })
        }).collect::<Vec<_>>(),
        "boss_relic_choices": state.boss_relic_choices.iter().enumerate().map(|(index, relic)| {
            json!({
                "index": index,
                "id": relic.id,
                "name": relic.name,
                "description": relic.description,
            })
        }).collect::<Vec<_>>(),
        "rest_options": state.rest_options,
        "event_choices": state.event_choices,
        "shop": {
            "purge_available": state.purge_available,
            "purge_cost": state.purge_cost,
        },
    })
}

fn parse_planner_response(
    response: &str,
    control: &AutoPlayControl,
    command_state: &CommandState,
    state: &NormalizedState,
    candidates: &[ActionCandidate],
) -> anyhow::Result<Option<AutoPlayAction>> {
    let parsed: PlannerResponse =
        serde_json::from_str(response.trim()).context("autoplay planner returned non-JSON")?;

    if parsed.schema_version != 1 {
        bail!(
            "autoplay planner returned unsupported schema_version {}",
            parsed.schema_version
        );
    }

    let [request] = parsed.actions.as_slice() else {
        bail!(
            "autoplay planner must return exactly one action, got {}",
            parsed.actions.len()
        );
    };

    if !candidates
        .iter()
        .any(|candidate| candidate.action_id == request.action_id && candidate.kind == request.kind)
    {
        bail!(
            "autoplay planner returned unavailable action_id {}",
            request.action_id
        );
    }

    let action = resolve_requested_action(
        control,
        &AutoPlaySession::default(),
        command_state,
        state,
        request,
    )
    .with_context(|| {
        format!(
            "autoplay planner action {} is not executable",
            request.action_id
        )
    })?;

    Ok(Some(action))
}

fn rejected_action_from_response(response: &str) -> Option<ActionRequestSummary> {
    let parsed = serde_json::from_str::<PlannerResponse>(response.trim()).ok()?;
    let request = parsed.actions.first()?;
    Some(ActionRequestSummary {
        kind: request.kind.clone(),
        action_id: request.action_id.clone(),
        target_index: request.target_index,
    })
}

fn fallback_action(
    control: &AutoPlayControl,
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    match state.screen_type.as_deref() {
        Some("COMBAT_REWARD") if control.allow_combat_rewards => {
            fallback_combat_reward_action(control, command_state)
        }
        Some("CARD_REWARD") if control.allow_card_rewards => {
            fallback_card_reward_action(command_state, state)
        }
        Some("BOSS_REWARD") if control.allow_boss_rewards => (!state.boss_relic_choices.is_empty()
            && command_state.has_command("choose"))
        .then_some(AutoPlayAction::Choose(0)),
        Some("REST") if control.allow_rest => fallback_rest_action(command_state, state),
        Some("EVENT") if control.allow_events => (!state.event_choices.is_empty()
            && command_state.has_command("choose"))
        .then_some(AutoPlayAction::Choose(0)),
        Some("SHOP_ROOM" | "SHOP_SCREEN") if control.allow_shop => command_state
            .has_command("leave")
            .then_some(AutoPlayAction::Leave),
        Some("MAP") if control.allow_map => (!command_state.choice_list.is_empty()
            && command_state.has_command("choose"))
        .then_some(AutoPlayAction::Choose(0)),
        Some("NONE") if control.allow_combat => fallback_combat_action(command_state, state),
        Some("GRID") if control.allow_selection_screens => command_state
            .has_command("choose")
            .then_some(AutoPlayAction::Choose(0))
            .or_else(|| {
                command_state
                    .has_command("confirm")
                    .then_some(AutoPlayAction::Proceed)
            }),
        Some("HAND_SELECT") if control.allow_selection_screens => command_state
            .has_command("choose")
            .then_some(AutoPlayAction::Choose(0))
            .or_else(|| {
                command_state
                    .has_command("confirm")
                    .then_some(AutoPlayAction::Proceed)
            }),
        _ => None,
    }
}

fn fallback_combat_reward_action(
    control: &AutoPlayControl,
    command_state: &CommandState,
) -> Option<AutoPlayAction> {
    if command_state.has_command("choose") {
        let index = command_state.choice_list.iter().position(|choice| {
            matches!(
                choice.as_str(),
                "gold" | "relic" | "stolen_gold" | "potion" | "emerald_key" | "sapphire_key"
            ) || (choice == "card" && control.allow_card_rewards)
        });
        if let Some(index) = index {
            return Some(AutoPlayAction::Choose(index));
        }
        if !command_state.choice_list.is_empty() {
            return Some(AutoPlayAction::Choose(0));
        }
    }

    if command_state.choice_list.is_empty() && command_state.has_command("proceed") {
        return Some(AutoPlayAction::Proceed);
    }

    None
}

fn fallback_card_reward_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if state.skip_available && command_state.has_command("skip") {
        return Some(AutoPlayAction::Skip);
    }

    if !state.card_reward_choices.is_empty() && command_state.has_command("choose") {
        return Some(AutoPlayAction::Choose(0));
    }

    None
}

fn fallback_rest_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if !command_state.has_command("choose") {
        return command_state
            .has_command("proceed")
            .then_some(AutoPlayAction::Proceed);
    }

    let rest_index = state
        .rest_options
        .iter()
        .position(|option| option == "rest");
    let smith_index = state
        .rest_options
        .iter()
        .position(|option| option == "smith");
    let hp_is_low = match (state.current_hp, state.max_hp) {
        (Some(current), Some(max)) if max > 0 => current * 2 < max,
        _ => false,
    };

    if hp_is_low {
        rest_index.or(smith_index).map(AutoPlayAction::Choose)
    } else {
        smith_index.or(rest_index).map(AutoPlayAction::Choose)
    }
}

fn fallback_combat_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if command_state.has_command("play") {
        let first_target = state.monsters.first().map(|monster| monster.index);
        if let Some((hand_index, target_index)) =
            state
                .hand
                .iter()
                .enumerate()
                .find_map(|(hand_index, card)| {
                    if !card.playable {
                        return None;
                    }

                    let target_index = if card.has_target { first_target } else { None };
                    if card.has_target && target_index.is_none() {
                        return None;
                    }

                    Some((hand_index, target_index))
                })
        {
            return Some(AutoPlayAction::Play {
                hand_index,
                target_index,
            });
        }
    }

    command_state
        .has_command("end")
        .then_some(AutoPlayAction::End)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autoplay::command_state::CommandState;
    use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
    use crate::locales::Locale;
    use serde_json::{Value, json};

    fn state(raw: Value) -> NormalizedState {
        NormalizedState::from_raw(&raw, &Locale::load("en"))
    }

    fn command_state(raw: &Value) -> CommandState {
        CommandState::from_raw(raw)
    }

    #[test]
    fn parses_llm_action_json_into_executable_action() {
        let raw = json!({
            "available_commands": ["choose", "skip"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "CARD_REWARD",
                "screen_state": {
                    "skip_available": true,
                    "cards": [
                        {"id": "Uppercut", "name": "Uppercut"},
                        {"id": "Anger", "name": "Anger"}
                    ]
                }
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = command_state(&raw);
        let state = state(raw);
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state,
        );

        let prompt = build_planner_prompt(
            &command_state,
            &state,
            &Locale::load("en"),
            false,
            &candidates,
            &[],
        )
        .unwrap();

        assert!(prompt.contains("=== Current State ==="));
        assert!(prompt.contains("\"language\": \"English\""));

        let action = parse_planner_response(
            r#"{
                "schema_version": 1,
                "actions": [{
                    "kind": "choose",
                    "action_id": "card_reward:1",
                    "label": "Anger",
                    "reason": "Cheap attack.",
                    "risk": ""
                }]
            }"#,
            &control,
            &command_state,
            &state,
            &candidates,
        )
        .unwrap();

        assert_eq!(action, Some(AutoPlayAction::Choose(1)));
    }

    #[test]
    fn rejects_unavailable_llm_action() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["Leave"]
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = command_state(&raw);
        let state = state(raw);
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state,
        );

        let error = parse_planner_response(
            r#"{
                "schema_version": 1,
                "actions": [{
                    "kind": "choose",
                    "action_id": "event:9",
                    "label": "Bad index",
                    "reason": "",
                    "risk": ""
                }]
            }"#,
            &control,
            &command_state,
            &state,
            &candidates,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unavailable action_id"));
    }

    #[test]
    fn rejects_non_json_llm_output() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["Leave"]
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = command_state(&raw);
        let state = state(raw);
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state,
        );

        let error = parse_planner_response(
            "推荐：do something",
            &control,
            &command_state,
            &state,
            &candidates,
        )
        .unwrap_err();

        assert!(error.to_string().contains("non-JSON"));
    }

    #[test]
    fn retry_prompt_includes_rejected_action_and_reason() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["Leave"]
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = command_state(&raw);
        let state = state(raw);
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state,
        );
        let rejections = vec![RejectedAttempt {
            attempt: 1,
            rejected_action: Some(ActionRequestSummary {
                kind: "choose".to_string(),
                action_id: "event:9".to_string(),
                target_index: None,
            }),
            reason: "autoplay planner returned unavailable action_id event:9".to_string(),
        }];

        let prompt = build_planner_prompt(
            &command_state,
            &state,
            &Locale::load("en"),
            false,
            &candidates,
            &rejections,
        )
        .unwrap();

        assert!(prompt.contains("\"rejected_attempts\""));
        assert!(prompt.contains("\"action_id\": \"event:9\""));
        assert!(prompt.contains("unavailable action_id"));
    }

    #[tokio::test]
    async fn mock_provider_plans_with_llm_json() {
        let raw = json!({
            "available_commands": ["choose", "skip"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "CARD_REWARD",
                "screen_state": {
                    "skip_available": true,
                    "cards": [
                        {"id": "Uppercut", "name": "Uppercut"}
                    ]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        let command_state = command_state(&raw);
        let state = state(raw);

        let action = plan_action(
            &LlmProvider::Mock,
            &mut control,
            &mut AutoPlaySession::default(),
            &command_state,
            &state,
            &Locale::load("en"),
            false,
        )
        .await
        .unwrap();

        assert_eq!(action, Some(AutoPlayAction::Skip));
    }

    #[test]
    fn fallback_event_with_multiple_choices_returns_first() {
        let raw = json!({
            "available_commands": ["choose"],
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["Fight", "Leave"]
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);

        let action = fallback_action(&control, &command_state, &state);
        assert_eq!(action, Some(AutoPlayAction::Choose(0)));
    }

    #[test]
    fn fallback_map_with_multiple_children_returns_first() {
        let raw = json!({
            "available_commands": ["choose"],
            "game_state": {
                "screen_type": "MAP",
                "choice_list": ["M", "?"]
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);

        let action = fallback_action(&control, &command_state, &state);
        assert_eq!(action, Some(AutoPlayAction::Choose(0)));
    }

    #[test]
    fn fallback_combat_reward_unknown_choice_returns_first() {
        let raw = json!({
            "available_commands": ["choose"],
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["unknown_reward"]
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);

        let action = fallback_action(&control, &command_state, &state);
        assert_eq!(action, Some(AutoPlayAction::Choose(0)));
    }

    #[test]
    fn fallback_hand_select_returns_first_choice() {
        let raw = json!({
            "available_commands": ["choose"],
            "game_state": {
                "screen_type": "HAND_SELECT",
                "choice_list": ["Strike", "Defend"]
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);

        let action = fallback_action(&control, &command_state, &state);
        assert_eq!(action, Some(AutoPlayAction::Choose(0)));
    }

    #[test]
    fn fallback_combat_uses_has_target_not_card_type() {
        let raw = json!({
            "available_commands": ["play", "end"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "player": {"energy": 3, "block": 0, "powers": []},
                    "hand": [
                        {"id": "Neutralize", "name": "Neutralize", "cost": 0, "type": "SKILL", "uuid": "neut-1", "has_target": true, "is_playable": true},
                        {"id": "Defend", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "def-1", "has_target": false, "is_playable": true}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);

        let action = fallback_action(&control, &command_state, &state);
        match action {
            Some(AutoPlayAction::Play { target_index, .. }) => {
                assert!(
                    target_index.is_some(),
                    "skill with has_target must include target"
                );
            }
            other => panic!("expected Play with target, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn retries_after_rejected_llm_action() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["retry_test_marker"]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        let command_state = command_state(&raw);
        let state = state(raw);

        let action = plan_action(
            &LlmProvider::Mock,
            &mut control,
            &mut AutoPlaySession::default(),
            &command_state,
            &state,
            &Locale::load("en"),
            false,
        )
        .await
        .unwrap();

        assert_eq!(action, Some(AutoPlayAction::Choose(0)));
    }

    #[tokio::test]
    async fn fallback_after_three_rejected_llm_actions() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["a", "b"]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        let command_state = command_state(&raw);
        let state = state(raw);

        let action = plan_action(
            &LlmProvider::Mock,
            &mut control,
            &mut AutoPlaySession::default(),
            &command_state,
            &state,
            &Locale::load("en"),
            false,
        )
        .await
        .unwrap();

        assert_eq!(action, Some(AutoPlayAction::Choose(0)));
    }

    #[test]
    fn try_deterministic_single_candidate_returns_action() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "screen_state": {
                    "choices": ["Proceed"]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state,
        );

        assert_eq!(candidates.len(), 1);
        let action = try_deterministic_action(
            &mut control,
            &mut AutoPlaySession::default(),
            &command_state,
            &state,
            &candidates,
        );
        assert_eq!(action, Some(AutoPlayAction::Choose(0)));
    }

    #[test]
    fn try_deterministic_multiple_candidates_returns_none() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "screen_state": {
                    "choices": ["Fight", "Leave"]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state,
        );

        assert_eq!(candidates.len(), 2);
        let action = try_deterministic_action(
            &mut control,
            &mut AutoPlaySession::default(),
            &command_state,
            &state,
            &candidates,
        );
        assert_eq!(action, None);
    }

    #[test]
    fn try_deterministic_combat_reward_uses_fallback() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["gold", "relic"]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state,
        );

        assert!(candidates.len() > 1);
        let action = try_deterministic_action(
            &mut control,
            &mut AutoPlaySession::default(),
            &command_state,
            &state,
            &candidates,
        );
        assert_eq!(action, Some(AutoPlayAction::Choose(0)));
    }

    #[test]
    fn fallback_rest_proceeds_when_choose_unavailable() {
        let raw = json!({
            "available_commands": ["proceed"],
            "game_state": {
                "screen_type": "REST",
                "screen_state": {
                    "rest_options": []
                }
            }
        });
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);

        let action = fallback_rest_action(&command_state, &state);
        assert_eq!(action, Some(AutoPlayAction::Proceed));
    }

    #[test]
    fn fallback_grid_proceeds_when_choose_unavailable() {
        let raw = json!({
            "available_commands": ["confirm"],
            "game_state": {
                "screen_type": "GRID",
                "choice_list": []
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);

        let action = fallback_action(&control, &command_state, &state);
        assert_eq!(action, Some(AutoPlayAction::Proceed));
    }

    #[test]
    fn fallback_hand_select_proceeds_when_choose_unavailable() {
        let raw = json!({
            "available_commands": ["confirm"],
            "game_state": {
                "screen_type": "HAND_SELECT",
                "choice_list": []
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);

        let action = fallback_action(&control, &command_state, &state);
        assert_eq!(action, Some(AutoPlayAction::Proceed));
    }

    #[test]
    fn try_deterministic_combat_reward_picks_potion_with_empty_slot() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["gold", "potion"],
                "potions": [
                    {"id": "Potion Slot", "name": "Potion Slot", "can_use": false, "can_discard": false, "description": ""},
                    {"id": "Potion Slot", "name": "Potion Slot", "can_use": false, "can_discard": false, "description": ""}
                ]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state,
        );

        let action = try_deterministic_action(
            &mut control,
            &mut AutoPlaySession::default(),
            &command_state,
            &state,
            &candidates,
        );
        assert_eq!(action, Some(AutoPlayAction::Choose(1)));
    }

    #[test]
    fn try_deterministic_combat_reward_with_full_slots_falls_through() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["potion", "card"],
                "potions": [
                    {"id": "Strength Potion", "name": "力量药水", "can_use": false, "can_discard": true, "description": ""}
                ]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        let command_state = CommandState::from_raw(&raw);
        let state = state(raw);
        let candidates = available_action_candidates(
            &control,
            &AutoPlaySession::default(),
            &command_state,
            &state,
        );

        // Potion in list but no empty slots — should fall through to LLM (returns None)
        let action = try_deterministic_action(
            &mut control,
            &mut AutoPlaySession::default(),
            &command_state,
            &state,
            &candidates,
        );
        assert_eq!(action, None);
    }
}
