use anyhow::{Context, bail};
use serde::Deserialize;
use serde_json::json;

use crate::autoplay::action::{
    ActionCandidate, ActionRequest, AutoPlayAction, available_action_candidates,
    resolve_requested_action,
};
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::AutoPlayControl;
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
    control: &AutoPlayControl,
    command_state: &CommandState,
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
) -> anyhow::Result<Option<AutoPlayAction>> {
    let candidates = available_action_candidates(control, command_state, state);
    if candidates.is_empty() {
        return Ok(None);
    }

    let effort = state
        .screen_type
        .as_deref()
        .map(Effort::from_screen_type)
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
                    Ok(action) => return Ok(action),
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

    let action =
        resolve_requested_action(control, command_state, state, request).with_context(|| {
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
        Some("EVENT") if control.allow_events => (state.event_choices.len() == 1
            && command_state.has_command("choose"))
        .then_some(AutoPlayAction::Choose(0)),
        Some("SHOP_SCREEN") if control.allow_shop => command_state
            .has_command("leave")
            .then_some(AutoPlayAction::Leave),
        Some("MAP") if control.allow_map => (command_state.choice_list.len() == 1
            && command_state.has_command("choose"))
        .then_some(AutoPlayAction::Choose(0)),
        Some("NONE") if control.allow_combat => fallback_combat_action(command_state, state),
        Some("GRID") if control.allow_selection_screens => {
            command_state
                .has_command("choose")
                .then_some(AutoPlayAction::Choose(0))
        }
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
                "gold" | "relic" | "potion" | "emerald_key" | "sapphire_key"
            ) || (choice == "card" && control.allow_card_rewards)
        });
        if let Some(index) = index {
            return Some(AutoPlayAction::Choose(index));
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
        return None;
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

                    let target_index = if card.card_type == "ATTACK" {
                        first_target
                    } else {
                        None
                    };
                    if card.card_type == "ATTACK" && target_index.is_none() {
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
    use crate::autoplay::control::AutoPlayControl;
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
        let candidates = available_action_candidates(&control, &command_state, &state);

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
        let candidates = available_action_candidates(&control, &command_state, &state);

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
        let candidates = available_action_candidates(&control, &command_state, &state);

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
        let candidates = available_action_candidates(&control, &command_state, &state);
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
        let control = AutoPlayControl::default_enabled();
        let command_state = command_state(&raw);
        let state = state(raw);

        let action = plan_action(
            &LlmProvider::Mock,
            &control,
            &command_state,
            &state,
            &Locale::load("en"),
            false,
        )
        .await
        .unwrap();

        assert_eq!(action, Some(AutoPlayAction::Skip));
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
        let control = AutoPlayControl::default_enabled();
        let command_state = command_state(&raw);
        let state = state(raw);

        let action = plan_action(
            &LlmProvider::Mock,
            &control,
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
                "choice_list": ["fallback_test_marker"]
            }
        });
        let control = AutoPlayControl::default_enabled();
        let command_state = command_state(&raw);
        let state = state(raw);

        let action = plan_action(
            &LlmProvider::Mock,
            &control,
            &command_state,
            &state,
            &Locale::load("en"),
            false,
        )
        .await
        .unwrap();

        assert_eq!(action, Some(AutoPlayAction::Choose(0)));
    }
}
