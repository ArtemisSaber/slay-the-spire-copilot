use crate::learning::session::LearningSession;

use super::{DeliberationOutcome, DeliberationResult, DeliberationState};

pub(super) fn finish(
    learning: &mut LearningSession,
    state: DeliberationState,
    run_id: &str,
    outcome: DeliberationOutcome,
) -> anyhow::Result<DeliberationResult> {
    learning.resolve_lesson_fallback(run_id)?;
    Ok(state.result(learning, outcome))
}
