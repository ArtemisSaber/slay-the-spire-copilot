use anyhow::{Context, bail};
use serde::Deserialize;
use serde_json::json;

use crate::autoplay::action::{
    ActionCandidate, ActionRequest, AutoPlayAction, resolve_requested_action,
};
use crate::autoplay::combat_adviser;
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
use crate::locales::Locale;
use crate::prompt;
use crate::state::NormalizedState;

#[derive(Debug, Deserialize)]
struct PlannerResponse {
    schema_version: u32,
    actions: Vec<ActionRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct RejectedAttempt {
    pub(crate) attempt: usize,
    pub(crate) rejected_action: Option<ActionRequestSummary>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct ActionRequestSummary {
    pub(crate) kind: String,
    pub(crate) action_id: String,
    pub(crate) target_index: Option<usize>,
}

pub(super) fn build_planner_prompt(
    session: &AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
    candidates: &[ActionCandidate],
    rejections: &[RejectedAttempt],
) -> anyhow::Result<String> {
    let localized_status_context = prompt::build_prompt(state, locale, shop_visited);

    let mut annotated_actions: Vec<ActionCandidate> = candidates.to_vec();
    if state.screen_type.as_ref().map(|st| st.as_str()) == Some("MAP")
        && !annotated_actions.is_empty()
    {
        let total = annotated_actions.len();
        for (i, c) in annotated_actions.iter_mut().enumerate() {
            let pos = crate::prompt::builder::position_label(i, total, locale);
            c.label = format!("({pos}) {}", c.label);
        }
    }

    let mut payload = json!({
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
        "state": state_summary(session, state),
        "available_commands": command_state.available_commands,
        "choice_list": command_state.choice_list,
        "available_actions": annotated_actions,
        "rejected_attempts": rejections,
    });

    if let Some(ranked) = combat_adviser::top_ranked_context(state) {
        payload["ranked_suggestions"] = ranked;
    }

    serde_json::to_string_pretty(&payload).context("failed to build autoplay planner prompt")
}

fn state_summary(session: &AutoPlaySession, state: &NormalizedState) -> serde_json::Value {
    let grid_purpose = if state.grid_for_upgrade {
        Some("upgrade")
    } else if state.grid_for_transform {
        Some("transform")
    } else if state.grid_for_purge {
        Some("purge")
    } else if session.pending_boss_relic_grid.is_some() {
        Some("relic")
    } else {
        None
    };

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
        "potions": state.potions.iter().map(|potion| {
            json!({
                "slot": potion.slot,
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
        "grid": if state.screen_type.as_ref().map(|st| st.as_str()) == Some("GRID") {
            json!({
                "purpose": grid_purpose,
                "num_cards": state.grid_num_cards,
                "triggered_by_relic": session.pending_boss_relic_grid,
                "selected_cards": state.grid_selected_cards.iter().enumerate().map(|(i, c)| {
                    json!({ "index": i, "id": c.id, "name": c.name })
                }).collect::<Vec<_>>(),
            })
        } else {
            json!(null)
        },
    })
}

pub(super) fn parse_planner_response(
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

pub(super) fn rejected_action_from_response(response: &str) -> Option<ActionRequestSummary> {
    let parsed = serde_json::from_str::<PlannerResponse>(response.trim())
        .map_err(|e| tracing::warn!("failed to parse planner response: {e}"))
        .ok()?;
    let request = parsed.actions.first()?;
    Some(ActionRequestSummary {
        kind: request.kind.clone(),
        action_id: request.action_id.clone(),
        target_index: request.target_index,
    })
}
