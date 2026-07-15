use crate::autoplay::action::{ActionCandidate, AutoPlayAction};
use crate::learning::telemetry::DecisionSource;
use crate::state::NormalizedState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlannedAction {
    pub(crate) action: AutoPlayAction,
    pub(crate) source: DecisionSource,
    pub(crate) selected_action_id: String,
    pub(crate) memory_ids_used: Vec<String>,
}

pub(super) fn planned_action(
    action: AutoPlayAction,
    source: DecisionSource,
    state: &NormalizedState,
    candidates: &[ActionCandidate],
    memory_ids_used: Vec<String>,
) -> PlannedAction {
    PlannedAction {
        selected_action_id: candidate_id(&action, state, candidates),
        action,
        source,
        memory_ids_used,
    }
}

fn candidate_id(
    action: &AutoPlayAction,
    state: &NormalizedState,
    candidates: &[ActionCandidate],
) -> String {
    let exact = match action {
        AutoPlayAction::Play { hand_index, .. } => state
            .hand
            .get(*hand_index)
            .and_then(|card| card.uuid.as_deref())
            .map(|uuid| format!("combat:play:{uuid}")),
        AutoPlayAction::Drink { slot_index, .. } => Some(format!("combat:potion:{slot_index}")),
        AutoPlayAction::End => Some("combat:end".to_string()),
        AutoPlayAction::Choose(index) => candidates
            .iter()
            .find(|candidate| candidate.action_id.ends_with(&format!(":{index}")))
            .map(|candidate| candidate.action_id.clone()),
        AutoPlayAction::Skip => by_kind(candidates, "skip"),
        AutoPlayAction::Proceed => by_kind(candidates, "proceed"),
        AutoPlayAction::Leave => by_kind(candidates, "leave"),
    };
    exact
        .filter(|id| {
            candidates
                .iter()
                .any(|candidate| candidate.action_id == *id)
        })
        .unwrap_or_else(|| "unmapped".to_string())
}

fn by_kind(candidates: &[ActionCandidate], kind: &str) -> Option<String> {
    candidates
        .iter()
        .find(|candidate| candidate.kind == kind)
        .map(|candidate| candidate.action_id.clone())
}
