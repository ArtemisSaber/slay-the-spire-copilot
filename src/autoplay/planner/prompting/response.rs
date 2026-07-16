use anyhow::{Context, anyhow, bail};
use serde::Deserialize;

use super::ActionSelectionSummary;
use crate::autoplay::action::{
    ActionCandidate, ActionRequest, AutoPlayAction, candidate_for_reference,
    resolve_requested_action,
};
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
use crate::state::NormalizedState;

#[derive(Debug, Deserialize)]
struct PlannerResponse {
    schema_version: u32,
    #[serde(default)]
    memory_ids_used: Vec<String>,
    actions: Vec<ActionSelection>,
}

#[derive(Debug, Deserialize)]
struct ActionSelection {
    #[serde(rename = "ref")]
    action_ref: String,
    #[serde(default)]
    target_index: Option<usize>,
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
    let parsed = deserialize_response(response)?;
    if parsed.schema_version != 2 {
        bail!(
            "autoplay planner returned unsupported schema_version {}",
            parsed.schema_version
        );
    }
    let [selection] = parsed.actions.as_slice() else {
        bail!(
            "autoplay planner must return exactly one action, got {}",
            parsed.actions.len()
        );
    };
    let Some(candidate) = candidate_for_reference(&selection.action_ref, candidates) else {
        bail!(
            "autoplay planner returned unknown action ref {}",
            selection.action_ref
        );
    };
    match (
        candidate.target_required == Some(true),
        selection.target_index,
    ) {
        (true, None) => bail!(
            "autoplay planner omitted target_index for targeted action ref {}",
            selection.action_ref
        ),
        (false, Some(_)) => bail!(
            "autoplay planner supplied target_index for untargeted action ref {}",
            selection.action_ref
        ),
        _ => {}
    }
    let request = ActionRequest {
        kind: candidate.kind.clone(),
        action_id: candidate.action_id.clone(),
        target_index: selection.target_index,
    };
    let action = resolve_requested_action(
        control,
        &AutoPlaySession::default(),
        command_state,
        state,
        &request,
    )
    .with_context(|| {
        format!(
            "autoplay planner action ref {} is not executable",
            selection.action_ref
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
        selected_action_id: candidate.action_id.clone(),
        memory_ids_used,
    })
}

pub(crate) fn rejected_action_from_response(response: &str) -> Option<ActionSelectionSummary> {
    let parsed = deserialize_response(response)
        .map_err(|error| tracing::warn!("failed to parse planner response: {error}"))
        .ok()?;
    let request = parsed.actions.first()?;
    Some(ActionSelectionSummary {
        action_ref: request.action_ref.clone(),
        target_index: request.target_index,
    })
}

fn deserialize_response(response: &str) -> anyhow::Result<PlannerResponse> {
    let json = serde_json::from_str(response.trim())
        .map_err(|error| anyhow!("autoplay planner returned invalid JSON syntax: {error}"))?;
    serde_json::from_value(json).map_err(|error| {
        anyhow!("autoplay planner returned valid JSON with an invalid schema: {error}")
    })
}
