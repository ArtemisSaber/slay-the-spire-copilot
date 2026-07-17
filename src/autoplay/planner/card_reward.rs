use serde_json::Value;

use crate::autoplay::action::ActionCandidate;
use crate::autoplay::control::AutoPlaySession;
use crate::llm::LlmProvider;
use crate::locales::Locale;
use crate::state::NormalizedState;

use super::prompting::structured_scenario;

mod comparison;
mod retry;
mod selection;
pub(super) use comparison::CardComparisonVerdict;
use comparison::compare_resulting_states;
#[cfg(test)]
pub(super) use comparison::{
    build_card_comparison_case_with_entropy, parse_card_comparison_response,
};
pub(super) use selection::CardCandidateSelection;
use selection::select_best_card;
#[cfg(test)]
pub(super) use selection::{build_card_candidate_prompt, parse_card_candidate_response};

#[allow(
    clippy::too_many_arguments,
    reason = "card reward planning keeps the current planner context explicit"
)]
pub(super) async fn plan_card_reward(
    provider: &LlmProvider,
    session: &AutoPlaySession,
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
    candidates: &[ActionCandidate],
    experience_context: Option<&Value>,
    allowed_memory_ids: &[String],
) -> anyhow::Result<super::PlannedAction> {
    let selection = select_best_card(
        provider,
        session,
        state,
        locale,
        shop_visited,
        candidates,
        experience_context,
        allowed_memory_ids,
    )
    .await?;
    let skip_candidate = candidates.iter().find(|candidate| candidate.kind == "skip");
    let comparison = if skip_candidate.is_some() {
        Some(
            compare_resulting_states(
                provider,
                session,
                state,
                locale,
                shop_visited,
                candidates,
                &selection,
                experience_context,
                allowed_memory_ids,
            )
            .await?,
        )
    } else {
        None
    };
    let take_card = comparison
        .as_ref()
        .is_none_or(|decision| decision.verdict == CardComparisonVerdict::PreferAddedCard);
    let (action, selected_action_id) = if take_card {
        (
            crate::autoplay::action::AutoPlayAction::Choose(selection.choice_index),
            candidates[selection.candidate_index].action_id.clone(),
        )
    } else {
        (
            crate::autoplay::action::AutoPlayAction::Skip,
            skip_candidate
                .expect("comparison requires a Skip candidate")
                .action_id
                .clone(),
        )
    };
    let mut memory_ids_used = selection.memory_ids_used;
    if let Some(comparison) = comparison {
        memory_ids_used.extend(comparison.memory_ids_used);
        memory_ids_used.sort();
        memory_ids_used.dedup();
    }
    Ok(super::PlannedAction {
        action,
        source: crate::learning::telemetry::DecisionSource::Llm,
        selected_action_id,
        memory_ids_used,
    })
}
