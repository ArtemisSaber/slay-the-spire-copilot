use anyhow::{Context, bail};
use serde::Deserialize;
use serde_json::json;

use crate::autoplay::action::{
    ActionCandidate, ActionRequest, AutoPlayAction, available_action_candidates,
    resolve_requested_action,
};
use crate::autoplay::combat_adviser;
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
use crate::llm::{Effort, LlmProvider};
use crate::locales::Locale;
use crate::prompt;
use crate::state::NormalizedState;

const MAX_LLM_ATTEMPTS: usize = 3;
const DETERMINISTIC_ACTION_DELAY: std::time::Duration = std::time::Duration::from_millis(1500);

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
    if state.screen_type.as_ref().map(|st| st.as_str()) == Some("COMBAT_REWARD")
        && session.last_combat_reward_floor != state.floor
    {
        session.skipped_combat_reward_potion = false;
        session.skipped_combat_reward_card = false;
        session.last_combat_reward_floor = state.floor;
    }

    let current_relic_ids: Vec<String> = state.relics.iter().map(|r| r.id.clone()).collect();
    for relic_id in &current_relic_ids {
        if !session.last_seen_relic_ids.contains(relic_id) {
            session.pending_boss_relic_grid = Some(relic_id.clone());
            break;
        }
    }
    session.last_seen_relic_ids = current_relic_ids;
    if session.pending_boss_relic_grid.is_some()
        && state.screen_type.as_ref().map(|st| st.as_str()) != Some("GRID")
    {
        session.pending_boss_relic_grid = None;
    }

    let candidates = available_action_candidates(control, session, command_state, state);
    if candidates.is_empty() {
        tracing::info!("autoplay no candidates, idling");
        return Ok(None);
    }

    if let Some(action) =
        try_deterministic_action(control, session, command_state, state, &candidates)
    {
        tracing::info!("autoplay deterministic {:?}", action);
        delay_before_deterministic_action().await;
        return Ok(Some(action));
    }

    if state.screen_type.as_ref().map(|st| st.as_str()) == Some("NONE")
        && let Some(action) = combat_adviser::try_kill_scan_action(state)
    {
        tracing::info!("autoplay kill_scan {:?}", action);
        delay_before_deterministic_action().await;
        return Ok(Some(action));
    }

    if state.screen_type.as_ref().map(|st| st.as_str()) == Some("SHOP_SCREEN") {
        session.last_shop_room_floor = state.floor;
    }

    let effort = state
        .screen_type
        .as_ref()
        .map(|st| Effort::from_screen_type(st.as_str(), !state.monsters.is_empty()))
        .unwrap_or(Effort::Medium);

    let mut rejections = vec![];
    for attempt in 1..=MAX_LLM_ATTEMPTS {
        let prompt = build_planner_prompt(
            session,
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
                        tracing::info!("autoplay LLM attempt={} {:?}", attempt, action);
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
                        if state.screen_type.as_ref().map(|st| st.as_str()) == Some("CARD_REWARD")
                            && matches!(&action, Some(AutoPlayAction::Skip))
                        {
                            session.skipped_combat_reward_card = true;
                        }
                        return Ok(action);
                    }
                    Err(e) => {
                        tracing::debug!("autoplay LLM attempt={} rejected: {e}", attempt,);
                        rejections.push(RejectedAttempt {
                            attempt,
                            rejected_action: rejected_action_from_response(&response),
                            reason: e.to_string(),
                        })
                    }
                }
            }
            Err(e) => {
                tracing::warn!("autoplay LLM attempt={} query error: {e}", attempt);
                rejections.push(RejectedAttempt {
                    attempt,
                    rejected_action: None,
                    reason: format!("LLM query failed: {e}"),
                })
            }
        }
    }

    let action = fallback_action(control, command_state, state);
    tracing::info!(
        "autoplay fallback after {} rejects {:?}",
        rejections.len(),
        action,
    );
    if action.is_some() {
        delay_before_deterministic_action().await;
    }
    Ok(action)
}

async fn delay_before_deterministic_action() {
    tokio::time::sleep(DETERMINISTIC_ACTION_DELAY).await;
}

fn potion_in_full_slots_was_rejected(
    candidates: &[ActionCandidate],
    command_state: &CommandState,
    state: &NormalizedState,
    action: &Option<AutoPlayAction>,
) -> Option<(usize, AutoPlayAction)> {
    if state.screen_type.as_ref().map(|st| st.as_str()) != Some("COMBAT_REWARD") {
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

        return Some(action);
    }

    // COMBAT_REWARD: collect unambiguous rewards deterministically. A key makes
    // the remaining reward choice strategic, so defer it to the LLM.
    if state.screen_type.as_ref().map(|st| st.as_str()) == Some("COMBAT_REWARD") {
        let has_potion = command_state.choice_list.iter().any(|c| c == "potion");
        let has_card = command_state.choice_list.iter().any(|c| c == "card");

        // 1. Gold is never a trade-off. Collect it before deciding among the
        // remaining rewards.
        if let Some(index) = command_state
            .choice_list
            .iter()
            .position(|choice| matches!(choice.as_str(), "gold" | "stolen_gold"))
            && command_state.has_command("choose")
        {
            return Some(AutoPlayAction::Choose(index));
        }

        // Taking a key instead of its linked relic is a run-level decision.
        // Do not let the first entry in choice_list decide it.
        if has_key_reward(&command_state.choice_list) {
            return None;
        }

        // Without a key alternative, collecting a relic is unambiguous.
        if let Some(index) = command_state
            .choice_list
            .iter()
            .position(|choice| choice == "relic")
            && command_state.has_command("choose")
        {
            return Some(AutoPlayAction::Choose(index));
        }

        // 2. Potion: deterministic if empty slots, LLM if full and not skipped.
        if has_potion && !session.skipped_combat_reward_potion {
            if state.empty_potion_slots > 0
                && let Some(index) = command_state.choice_list.iter().position(|c| c == "potion")
            {
                return Some(AutoPlayAction::Choose(index));
            }
            return None;
        }

        // 3. Card: always pick (enters CARD_REWARD where LLM decides pick/skip).
        if has_card
            && !session.skipped_combat_reward_card
            && let Some(index) = command_state.choice_list.iter().position(|c| c == "card")
        {
            return Some(AutoPlayAction::Choose(index));
        }

        // 4. Safety net: any unknown leftover choice.
        if !command_state.choice_list.is_empty() && command_state.has_command("choose") {
            return Some(AutoPlayAction::Choose(0));
        }

        // 5. Nothing left — proceed.
        if command_state.has_command("proceed") {
            return Some(AutoPlayAction::Proceed);
        }
        return None;
    }

    None
}

fn build_planner_prompt(
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

fn fallback_action(
    control: &AutoPlayControl,
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    match state.screen_type.as_ref().map(|st| st.as_str()) {
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
        Some("CHEST") if control.allow_selection_screens => command_state
            .has_command("choose")
            .then_some(AutoPlayAction::Choose(0))
            .or_else(|| {
                command_state
                    .has_command("proceed")
                    .then_some(AutoPlayAction::Proceed)
            }),
        Some("COMPLETE") if control.allow_selection_screens => command_state
            .has_command("proceed")
            .then_some(AutoPlayAction::Proceed),
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
    _control: &AutoPlayControl,
    command_state: &CommandState,
) -> Option<AutoPlayAction> {
    if command_state.has_command("choose") {
        if let Some(index) = command_state
            .choice_list
            .iter()
            .position(|choice| matches!(choice.as_str(), "gold" | "stolen_gold"))
        {
            return Some(AutoPlayAction::Choose(index));
        }
        if has_key_reward(&command_state.choice_list) {
            return None;
        }
        if let Some(index) = command_state
            .choice_list
            .iter()
            .position(|choice| choice == "relic")
        {
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

fn has_key_reward(choice_list: &[String]) -> bool {
    choice_list
        .iter()
        .any(|choice| matches!(choice.as_str(), "emerald_key" | "sapphire_key"))
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
#[path = "tests/planner_tests/mod.rs"]
mod tests;
