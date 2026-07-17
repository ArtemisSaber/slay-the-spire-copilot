use crate::learning::session::{CriticDraft, LearningSession};

use super::{DeliberationOutcome, DeliberationResult, DeliberationState};

pub(super) fn commit(
    learning: &mut LearningSession,
    draft: CriticDraft,
    state: DeliberationState,
) -> anyhow::Result<DeliberationResult> {
    let mut ingest = learning.commit_critic_draft(draft)?;
    ingest.rejected_lessons += state.rejected_lessons;
    Ok(DeliberationResult {
        ingest,
        api_calls: state.api_calls,
        outcome: DeliberationOutcome::Approved,
    })
}
