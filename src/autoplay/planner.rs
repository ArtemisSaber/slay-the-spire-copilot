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
use crate::state::NormalizedState;

#[derive(Debug, Deserialize)]
struct PlannerResponse {
    schema_version: u32,
    actions: Vec<ActionRequest>,
}

pub async fn plan_action(
    provider: &LlmProvider,
    control: &AutoPlayControl,
    command_state: &CommandState,
    state: &NormalizedState,
) -> anyhow::Result<Option<AutoPlayAction>> {
    let candidates = available_action_candidates(control, command_state, state);
    if candidates.is_empty() {
        return Ok(None);
    }

    let prompt = build_planner_prompt(command_state, state, &candidates)?;
    let effort = state
        .screen_type
        .as_deref()
        .map(Effort::from_screen_type)
        .unwrap_or(Effort::Medium);
    let response = provider.query_autoplay_action(&prompt, effort).await?;

    parse_planner_response(&response, control, command_state, state, &candidates)
}

fn build_planner_prompt(
    command_state: &CommandState,
    state: &NormalizedState,
    candidates: &[ActionCandidate],
) -> anyhow::Result<String> {
    let payload = json!({
        "task": "Choose exactly one action_id from available_actions. Return strict JSON only.",
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
        "state": state_summary(state),
        "available_commands": command_state.available_commands,
        "choice_list": command_state.choice_list,
        "available_actions": candidates,
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

        let action = plan_action(&LlmProvider::Mock, &control, &command_state, &state)
            .await
            .unwrap();

        assert_eq!(action, Some(AutoPlayAction::Skip));
    }
}
