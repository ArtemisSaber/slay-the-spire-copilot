use serde_json::Value;

use crate::learning::session::{CriticDraft, LearningSession};
use crate::llm::LlmProvider;
use crate::locales::Locale;

use super::{DeliberationState, MAX_API_CALLS, MAX_ROLE_ATTEMPTS, Stage, support};

#[allow(
    clippy::too_many_arguments,
    reason = "the proposer stage keeps its immutable evidence and shared call state explicit"
)]
pub(super) async fn propose(
    learning: &LearningSession,
    provider: &LlmProvider,
    locale: &Locale,
    base_report_prompt: &str,
    run_id: &str,
    review_rejection: Option<&Value>,
    state: &mut DeliberationState,
) -> Stage<CriticDraft> {
    let mut retry_feedback = vec![];
    let mut ordinary_failures = 0;
    loop {
        if state.api_calls >= MAX_API_CALLS {
            return Stage::BudgetExhausted;
        }
        let Some(prompt) = learning.build_critic_prompt_with_feedback(
            base_report_prompt,
            run_id,
            review_rejection,
            &retry_feedback,
        ) else {
            return Stage::Failed;
        };
        state.api_calls += 1;
        let ordinary_failure = match provider.query_learning_postmortem(&prompt, locale).await {
            Ok(response) => match learning.prepare_critic_response(&response, run_id) {
                Ok(draft) => return Stage::Complete(draft),
                Err(rejection) => {
                    let ordinary = !rejection.abstained();
                    state.remember_rejection(&rejection);
                    push_feedback(&mut retry_feedback, rejection.feedback().to_string());
                    ordinary
                }
            },
            Err(error) => {
                push_feedback(
                    &mut retry_feedback,
                    support::error_feedback("proposer query failed", &error),
                );
                true
            }
        };
        ordinary_failures = if ordinary_failure {
            ordinary_failures + 1
        } else {
            0
        };
        if ordinary_failures >= MAX_ROLE_ATTEMPTS {
            return Stage::Failed;
        }
    }
}

fn push_feedback(feedback: &mut Vec<String>, message: String) {
    if feedback.len() == MAX_ROLE_ATTEMPTS {
        feedback.remove(0);
    }
    feedback.push(message);
}
