use super::GameRuntime;
use crate::autoplay::planner::PlannedAction;
use crate::learning::action::SemanticAction;
use crate::learning::session::{ExecutedPlan, PreparedMemory};
use crate::learning::telemetry::RecordedRankedAction;
use crate::state::NormalizedState;

impl GameRuntime {
    pub(super) fn prepare_learning(&self, state: &NormalizedState) -> Option<PreparedMemory> {
        let tags = crate::autoplay::combat_adviser::ranker_tags(state);
        self.learning.prepare(state, &tags)
    }

    pub(super) fn record_learning_execution(
        &mut self,
        state: &NormalizedState,
        planned: &PlannedAction,
        prepared: Option<&PreparedMemory>,
        available_semantic_actions: Vec<SemanticAction>,
        ranked_suggestions: Vec<RecordedRankedAction>,
    ) {
        let Some(prepared) = prepared else {
            return;
        };
        let executed = ExecutedPlan {
            action: planned.action.clone(),
            source: planned.source,
            selected_action_id: planned.selected_action_id.clone(),
            available_semantic_actions,
            ranked_suggestions,
            memory_ids_used: planned.memory_ids_used.clone(),
        };
        if let Some(event) =
            self.learning
                .record_executed(self.journal.run_id(), state, &executed, prepared)
        {
            self.journal.log_learning_event(&event);
        }
    }
}
