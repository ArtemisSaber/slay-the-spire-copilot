use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::learning::config::{MemoryConfig, MemoryMode};
use crate::learning::deliberation::{DeliberationOutcome, deliberate_lesson};
use crate::learning::session::{LearningSession, SessionProvenance};
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::store::KnowledgeStore;
use crate::llm::LlmProvider;
use crate::locales::Locale;

use super::support::decision_case;

fn session(temp: &std::path::Path) -> (LearningSession, String) {
    let case = decision_case("run-a", 1);
    let decision_id = case.decision_id.clone();
    let store = KnowledgeStore::new(temp);
    store.append_cases(std::slice::from_ref(&case)).unwrap();
    let learning = LearningSession::new(
        MemoryConfig {
            mode: MemoryMode::Collect,
            ..MemoryConfig::default()
        },
        store,
        KnowledgeSnapshot::build(vec![case], &[]).unwrap(),
        SessionProvenance {
            locale: "en".into(),
            model_profile_sha256: "model".into(),
            compatibility_sha256: "mods".into(),
            rules_sha256: "rules".into(),
            synthetic_input: false,
        },
    );
    (learning, decision_id)
}

fn abstention() -> String {
    serde_json::json!({
        "schema_version": 3,
        "report_markdown": "# Review\n\nThe run contains several possible improvements.",
        "result": "no_lesson",
        "lesson": null,
        "rejected_lesson_analysis": null
    })
    .to_string()
}

fn proposal(decision_id: &str) -> String {
    serde_json::json!({
        "schema_version": 3,
        "report_markdown": "# Review\n\nA fact-grounded run review.",
        "result": "lesson",
        "lesson": {
            "text": "Preserve HP before extending setup.",
            "applies_when": "The recorded tactical conditions recur.",
            "expected_effect": "This may improve survival in comparable fights.",
            "evidence": [{
                "run_id": "run-a",
                "decision_ids": [decision_id],
                "observed_chain": "The cited action was followed by the recorded outcome."
            }],
            "uncertainty": "The unchosen action was not observed.",
            "confidence_millis": 700
        },
        "rejected_lesson_analysis": null
    })
    .to_string()
}

#[tokio::test]
async fn proposer_abstention_is_rejected_and_retried() {
    let temp = tempfile::tempdir().unwrap();
    let (mut learning, decision_id) = session(temp.path());
    let proposer_calls = Arc::new(AtomicUsize::new(0));
    let proposer_counter = proposer_calls.clone();
    let provider = LlmProvider::scripted(move |system, prompt, _effort| {
        if system.contains("LESSON_FACT_REVIEWER_V4") {
            return Ok(crate::llm::mock::mock_lesson_fact_review_response(prompt));
        }
        let call = proposer_counter.fetch_add(1, Ordering::SeqCst);
        if call == 0 {
            return Ok(abstention());
        }
        assert!(prompt.contains("retry_context"));
        assert!(prompt.contains("must return a lesson candidate"));
        Ok(proposal(&decision_id))
    });

    let result = deliberate_lesson(
        &mut learning,
        &provider,
        &Locale::load("en"),
        "base report",
        "run-a",
    )
    .await
    .unwrap();

    assert_eq!(result.outcome, DeliberationOutcome::Approved);
    assert_eq!(result.api_calls, 3);
    assert_eq!(proposer_calls.load(Ordering::SeqCst), 2);
    assert_eq!(result.ingest.accepted_lessons, 1);
}

#[tokio::test]
async fn repeated_proposer_abstentions_exhaust_the_full_budget_before_failing_safe() {
    let temp = tempfile::tempdir().unwrap();
    let (mut learning, _) = session(temp.path());
    let provider = LlmProvider::scripted(|system, _prompt, _effort| {
        assert!(!system.contains("LESSON_FACT_REVIEWER_V4"));
        Ok(abstention())
    });

    let result = deliberate_lesson(
        &mut learning,
        &provider,
        &Locale::load("en"),
        "base report",
        "run-a",
    )
    .await
    .unwrap();

    assert_eq!(result.outcome, DeliberationOutcome::BudgetExhausted);
    assert_eq!(result.api_calls, 16);
    assert_eq!(provider.recorded_requests().len(), 16);
    assert_eq!(result.ingest.accepted_lessons, 0);
    assert!(learning.snapshot().lessons.is_empty());
}

#[tokio::test]
async fn already_reviewed_run_uses_report_only_call_without_proposer_abstention() {
    let temp = tempfile::tempdir().unwrap();
    let (mut learning, decision_id) = session(temp.path());
    let ingest = learning
        .ingest_critic_response(&proposal(&decision_id), "run-a")
        .unwrap();
    assert_eq!(ingest.accepted_lessons, 1);

    let provider = LlmProvider::scripted(|system, _prompt, effort| {
        assert_eq!(effort.as_str(), "heavy");
        assert!(!system.contains("LEARNING_CRITIC_ENVELOPE_V3"));
        assert!(!system.contains("LESSON_FACT_REVIEWER_V4"));
        Ok("# Refreshed Review\n\nNo duplicate lesson was requested.".into())
    });

    let result = deliberate_lesson(
        &mut learning,
        &provider,
        &Locale::load("en"),
        "base report",
        "run-a",
    )
    .await
    .unwrap();

    assert_eq!(result.outcome, DeliberationOutcome::ReportOnly);
    assert_eq!(result.api_calls, 1);
    assert_eq!(result.ingest.accepted_lessons, 0);
    assert_eq!(
        result.ingest.report_markdown,
        "# Refreshed Review\n\nNo duplicate lesson was requested."
    );
    assert_eq!(learning.snapshot().lessons.len(), 1);
}
