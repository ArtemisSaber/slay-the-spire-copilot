use crate::autoplay::action::{AutoPlayAction, available_action_candidates};
use crate::autoplay::combat_adviser;
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
use crate::learning::telemetry::DecisionSource;
use crate::llm::{Effort, LlmProvider};
use crate::locales::Locale;
use crate::state::NormalizedState;

mod decision;
mod deterministic;
mod fallback;
mod prompting;

pub(crate) use decision::PlannedAction;
use decision::planned_action;
use deterministic::{potion_in_full_slots_was_rejected, try_deterministic_action};
use fallback::fallback_action;
#[cfg(test)]
use fallback::fallback_rest_action;
#[cfg(test)]
use prompting::ActionSelectionSummary;
use prompting::{
    RejectedAttempt, build_planner_prompt_with_memory, parse_planner_response_with_memory,
    rejected_action_from_response,
};
#[cfg(test)]
use prompting::{build_planner_prompt, parse_planner_response};

const MAX_LLM_ATTEMPTS: usize = 3;
const DETERMINISTIC_ACTION_DELAY: std::time::Duration = std::time::Duration::from_millis(1500);

#[cfg(test)]
pub async fn plan_action(
    provider: &LlmProvider,
    control: &mut AutoPlayControl,
    session: &mut AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
) -> anyhow::Result<Option<AutoPlayAction>> {
    plan_action_with_memory(
        provider,
        control,
        session,
        command_state,
        state,
        locale,
        shop_visited,
        None,
        &[],
    )
    .await
    .map(|planned| planned.map(|planned| planned.action))
}

#[allow(
    clippy::too_many_arguments,
    reason = "planner orchestration keeps its existing runtime dependencies explicit"
)]
pub(crate) async fn plan_action_with_memory(
    provider: &LlmProvider,
    control: &mut AutoPlayControl,
    session: &mut AutoPlaySession,
    command_state: &CommandState,
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
    experience_context: Option<&serde_json::Value>,
    retrieved_memory_ids: &[String],
) -> anyhow::Result<Option<PlannedAction>> {
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
        return Ok(Some(planned_action(
            action,
            DecisionSource::Deterministic,
            state,
            &candidates,
            vec![],
        )));
    }

    if state.screen_type.as_ref().map(|st| st.as_str()) == Some("NONE")
        && let Some(action) = combat_adviser::try_kill_scan_action(state)
    {
        tracing::info!("autoplay kill_scan {:?}", action);
        delay_before_deterministic_action().await;
        return Ok(Some(planned_action(
            action,
            DecisionSource::KillScan,
            state,
            &candidates,
            vec![],
        )));
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
        let prompt = build_planner_prompt_with_memory(
            session,
            command_state,
            state,
            locale,
            shop_visited,
            &candidates,
            &rejections,
            experience_context,
        )?;
        match provider
            .query_autoplay_action(&prompt, effort, locale)
            .await
        {
            Ok(response) => {
                match parse_planner_response_with_memory(
                    &response,
                    control,
                    command_state,
                    state,
                    &candidates,
                    retrieved_memory_ids,
                ) {
                    Ok(parsed) => {
                        tracing::info!("autoplay LLM attempt={} {:?}", attempt, parsed.action);
                        if let Some((potion_index, AutoPlayAction::Choose(chosen))) =
                            potion_in_full_slots_was_rejected(
                                &candidates,
                                command_state,
                                state,
                                &Some(parsed.action.clone()),
                            )
                            && chosen != potion_index
                        {
                            session.skipped_combat_reward_potion = true;
                        }
                        if state.screen_type.as_ref().map(|st| st.as_str()) == Some("CARD_REWARD")
                            && matches!(&parsed.action, AutoPlayAction::Skip)
                        {
                            session.skipped_combat_reward_card = true;
                        }
                        return Ok(Some(PlannedAction {
                            action: parsed.action,
                            source: DecisionSource::Llm,
                            selected_action_id: parsed.selected_action_id,
                            memory_ids_used: parsed.memory_ids_used,
                        }));
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
    Ok(action
        .map(|action| planned_action(action, DecisionSource::Fallback, state, &candidates, vec![])))
}

async fn delay_before_deterministic_action() {
    tokio::time::sleep(DETERMINISTIC_ACTION_DELAY).await;
}

#[cfg(test)]
#[path = "tests/planner_tests/mod.rs"]
mod tests;
