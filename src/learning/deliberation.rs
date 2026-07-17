use serde_json::{Value, json};

use crate::learning::fact_review::{FactReview, FactReviewVerdict, parse_fact_review};
use crate::learning::session::{CriticDraft, CriticDraftRejection, CriticIngest, LearningSession};
use crate::llm::LlmProvider;
use crate::locales::Locale;

mod approval;
mod fallback;
mod proposer;
mod report_only;
mod support;

const MAX_API_CALLS: usize = 16;
const MAX_ROLE_ATTEMPTS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeliberationOutcome {
    Approved,
    ReportOnly,
    BudgetExhausted,
    Failed,
}

impl DeliberationOutcome {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::ReportOnly => "report_only",
            Self::BudgetExhausted => "budget_exhausted",
            Self::Failed => "failed",
        }
    }
}

pub(crate) struct DeliberationResult {
    pub(crate) ingest: CriticIngest,
    pub(crate) api_calls: usize,
    pub(crate) outcome: DeliberationOutcome,
}

#[derive(Default)]
struct DeliberationState {
    api_calls: usize,
    latest_report: String,
    response_valid: bool,
    rejected_lessons: usize,
}

enum Stage<T> {
    Complete(T),
    BudgetExhausted,
    Failed,
}

enum ReviewAction {
    Approve,
    Reject(FactReview),
}

pub(crate) async fn deliberate_lesson(
    learning: &mut LearningSession,
    provider: &LlmProvider,
    locale: &Locale,
    base_report_prompt: &str,
    run_id: &str,
) -> anyhow::Result<DeliberationResult> {
    if learning.critic_is_report_only(run_id) {
        return Ok(report_only::deliberate(learning, provider, locale, base_report_prompt).await);
    }
    let mut state = DeliberationState::default();
    let mut review_rejection: Option<Value> = None;
    let deterministic_report = support::deterministic_report(base_report_prompt, locale);
    loop {
        let draft = match proposer::propose(
            learning,
            provider,
            locale,
            base_report_prompt,
            run_id,
            review_rejection.as_ref(),
            &mut state,
        )
        .await
        {
            Stage::Complete(draft) => draft,
            Stage::BudgetExhausted => {
                return fallback::finish(
                    learning,
                    state,
                    run_id,
                    DeliberationOutcome::BudgetExhausted,
                );
            }
            Stage::Failed => {
                return fallback::finish(learning, state, run_id, DeliberationOutcome::Failed);
            }
        };
        state.remember_draft(&draft);
        match review(
            learning,
            provider,
            deterministic_report,
            run_id,
            &draft,
            &mut state,
        )
        .await
        {
            Stage::Complete(ReviewAction::Approve) => {
                return approval::commit(learning, draft, state);
            }
            Stage::Complete(ReviewAction::Reject(review)) => {
                state.rejected_lessons += 1;
                review_rejection = Some(json!({
                    "previous_lesson": draft.candidate_json(),
                    "reviewer_feedback": review,
                }));
            }
            Stage::BudgetExhausted => {
                return fallback::finish(
                    learning,
                    state,
                    run_id,
                    DeliberationOutcome::BudgetExhausted,
                );
            }
            Stage::Failed => {
                return fallback::finish(learning, state, run_id, DeliberationOutcome::Failed);
            }
        }
    }
}

async fn review(
    learning: &LearningSession,
    provider: &LlmProvider,
    deterministic_report: &str,
    run_id: &str,
    draft: &CriticDraft,
    state: &mut DeliberationState,
) -> Stage<ReviewAction> {
    let mut retry_feedback = vec![];
    for _ in 0..MAX_ROLE_ATTEMPTS {
        if state.api_calls >= MAX_API_CALLS {
            return Stage::BudgetExhausted;
        }
        let Some(prompt) =
            learning.build_fact_review_prompt(deterministic_report, run_id, draft, &retry_feedback)
        else {
            return Stage::Failed;
        };
        state.api_calls += 1;
        let required_claims = draft.review_claims();
        let parsed = match provider.query_lesson_fact_review(&prompt).await {
            Ok(response) => {
                parse_fact_review(&response, draft.allowed_decision_ids(), &required_claims)
            }
            Err(error) => {
                retry_feedback.push(support::error_feedback(
                    "fact reviewer query failed",
                    &error,
                ));
                continue;
            }
        };
        match parsed {
            Ok(review) if review.verdict == FactReviewVerdict::Approve => {
                return Stage::Complete(ReviewAction::Approve);
            }
            Ok(review) => return Stage::Complete(ReviewAction::Reject(review)),
            Err(error) => {
                retry_feedback.push(support::error_feedback(
                    "fact reviewer response rejected",
                    &error,
                ));
            }
        }
    }
    Stage::Failed
}

impl DeliberationState {
    fn remember_draft(&mut self, draft: &CriticDraft) {
        self.latest_report = draft.report_markdown().to_string();
        self.response_valid = true;
    }

    fn remember_rejection(&mut self, rejection: &CriticDraftRejection) {
        if rejection.response_valid() {
            self.latest_report = rejection.report_markdown().to_string();
            self.response_valid = true;
        }
        self.rejected_lessons += rejection.rejected_lessons();
    }

    fn result(
        &self,
        learning: &LearningSession,
        outcome: DeliberationOutcome,
    ) -> DeliberationResult {
        DeliberationResult {
            ingest: learning.deliberation_ingest(
                self.latest_report.clone(),
                self.response_valid,
                self.rejected_lessons,
            ),
            api_calls: self.api_calls,
            outcome,
        }
    }
}
