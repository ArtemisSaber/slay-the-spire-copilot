use anyhow::{Context, bail};
use serde::Deserialize;

use super::ActionRequestSummary;
use crate::autoplay::action::{
    ActionCandidate, ActionRequest, AutoPlayAction, resolve_requested_action,
};
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
use crate::state::NormalizedState;

#[derive(Debug, Deserialize)]
struct PlannerResponse {
    schema_version: u32,
    #[serde(default)]
    memory_ids_used: Vec<String>,
    actions: Vec<ActionRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedPlannerResponse {
    pub(crate) action: AutoPlayAction,
    pub(crate) selected_action_id: String,
    pub(crate) memory_ids_used: Vec<String>,
}

#[cfg(test)]
pub(crate) fn parse_planner_response(
    response: &str,
    control: &AutoPlayControl,
    command_state: &CommandState,
    state: &NormalizedState,
    candidates: &[ActionCandidate],
) -> anyhow::Result<Option<AutoPlayAction>> {
    parse_planner_response_with_memory(response, control, command_state, state, candidates, &[])
        .map(|parsed| Some(parsed.action))
}

pub(crate) fn parse_planner_response_with_memory(
    response: &str,
    control: &AutoPlayControl,
    command_state: &CommandState,
    state: &NormalizedState,
    candidates: &[ActionCandidate],
    allowed_memory_ids: &[String],
) -> anyhow::Result<ParsedPlannerResponse> {
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
    let mut memory_ids_used: Vec<_> = parsed
        .memory_ids_used
        .into_iter()
        .filter(|id| allowed_memory_ids.contains(id))
        .collect();
    memory_ids_used.sort();
    memory_ids_used.dedup();
    Ok(ParsedPlannerResponse {
        action,
        selected_action_id: request.action_id.clone(),
        memory_ids_used,
    })
}

pub(crate) fn rejected_action_from_response(response: &str) -> Option<ActionRequestSummary> {
    let parsed = serde_json::from_str::<PlannerResponse>(response.trim())
        .map_err(|error| tracing::warn!("failed to parse planner response: {error}"))
        .ok()?;
    let request = parsed.actions.first()?;
    Some(ActionRequestSummary {
        kind: request.kind.clone(),
        action_id: request.action_id.clone(),
        target_index: request.target_index,
    })
}
