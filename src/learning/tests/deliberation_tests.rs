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
    (
        LearningSession::new(
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
        ),
        decision_id,
    )
}

fn proposal(decision_id: &str, text: &str) -> String {
    serde_json::json!({
        "schema_version": 3,
        "report_markdown": "# Review\n\nA fact-grounded run review.",
        "result": "lesson",
        "lesson": {
            "text": text,
            "applies_when": "The recorded tactical conditions recur.",
            "expected_effect": "This may improve survival in comparable fights.",
            "evidence": [{
                "run_id": "run-a",
                "decision_ids": [decision_id],
                "observed_chain": "The cited action was followed by the recorded HP and run outcome."
            }],
            "uncertainty": "The unchosen action was not observed.",
            "confidence_millis": 700
        },
        "rejected_lesson_analysis": null
    })
    .to_string()
}

fn approval(prompt: &str) -> String {
    crate::llm::mock::mock_lesson_fact_review_response(prompt)
}

fn rejection(prompt: &str) -> String {
    let mut review: serde_json::Value = serde_json::from_str(&approval(prompt)).unwrap();
    review["verdict"] = "reject".into();
    review["feedback"] =
        "Correct the HP claim and avoid asserting an unobserved causal result.".into();
    review["checks"][0]["status"] = "unsupported".into();
    review["checks"][0]["refs"] = serde_json::json!([]);
    review.to_string()
}

#[tokio::test]
async fn approved_lesson_uses_two_heavy_calls_then_commits() {
    let temp = tempfile::tempdir().unwrap();
    let (mut learning, decision_id) = session(temp.path());
    let provider = LlmProvider::scripted(move |system, prompt, effort| {
        assert_eq!(effort.as_str(), "heavy");
        if system.contains("LESSON_FACT_REVIEWER_V3") {
            assert!(prompt.starts_with("LESSON_FACT_REVIEWER_V3\n"));
            let payload: serde_json::Value =
                serde_json::from_str(prompt.split_once('\n').unwrap().1).unwrap();
            let check = payload.pointer("/output_contract/checks/0").unwrap();
            assert_eq!(check.as_object().unwrap().len(), 3);
            Ok(approval(prompt))
        } else {
            Ok(proposal(
                &decision_id,
                "Preserve HP before extending setup.",
            ))
        }
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
    assert_eq!(result.api_calls, 2);
    assert_eq!(result.ingest.accepted_lessons, 1);
    assert_eq!(learning.snapshot().lessons.len(), 1);
    assert_eq!(provider.recorded_requests().len(), 2);
}

#[tokio::test]
async fn rejected_lesson_is_revised_with_fact_reviewer_feedback() {
    let temp = tempfile::tempdir().unwrap();
    let (mut learning, decision_id) = session(temp.path());
    let proposer_calls = Arc::new(AtomicUsize::new(0));
    let reviewer_calls = Arc::new(AtomicUsize::new(0));
    let proposer_counter = proposer_calls.clone();
    let reviewer_counter = reviewer_calls.clone();
    let expected_feedback = "Correct the HP claim and avoid asserting an unobserved causal result.";
    let provider = LlmProvider::scripted(move |system, prompt, _effort| {
        if system.contains("LESSON_FACT_REVIEWER_V3") {
            let call = reviewer_counter.fetch_add(1, Ordering::SeqCst);
            return Ok(if call == 0 {
                rejection(prompt)
            } else {
                approval(prompt)
            });
        }
        let call = proposer_counter.fetch_add(1, Ordering::SeqCst);
        if call == 1 {
            assert!(prompt.contains("reviewer_feedback"));
            assert!(prompt.contains(expected_feedback));
        }
        Ok(proposal(
            &decision_id,
            if call == 0 {
                "The selected action preserved HP."
            } else {
                "When immediate HP loss is recorded, prioritize reducing near-term pressure."
            },
        ))
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
    assert_eq!(result.api_calls, 4);
    assert_eq!(proposer_calls.load(Ordering::SeqCst), 2);
    assert_eq!(reviewer_calls.load(Ordering::SeqCst), 2);
    assert_eq!(learning.snapshot().lessons.len(), 1);
}

#[tokio::test]
async fn repeated_rejections_stop_at_sixteen_calls_without_a_lesson() {
    let temp = tempfile::tempdir().unwrap();
    let (mut learning, decision_id) = session(temp.path());
    let provider = LlmProvider::scripted(move |system, prompt, _effort| {
        if system.contains("LESSON_FACT_REVIEWER_V3") {
            Ok(rejection(prompt))
        } else {
            Ok(proposal(&decision_id, "The selected action preserved HP."))
        }
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
async fn invalid_reviewer_outputs_retry_the_reviewer_three_times() {
    let temp = tempfile::tempdir().unwrap();
    let (mut learning, decision_id) = session(temp.path());
    let reviewer_calls = Arc::new(AtomicUsize::new(0));
    let reviewer_counter = reviewer_calls.clone();
    let provider = LlmProvider::scripted(move |system, prompt, _effort| {
        if system.contains("LESSON_FACT_REVIEWER_V3") {
            let call = reviewer_counter.fetch_add(1, Ordering::SeqCst);
            return Ok(if call == 0 {
                "not json".into()
            } else if call == 1 {
                r#"{"schema_version":3,"verdict":"approve","feedback":null,"checks":[]}"#.into()
            } else {
                approval(prompt)
            });
        }
        Ok(proposal(
            &decision_id,
            "Preserve HP before extending setup.",
        ))
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
    assert_eq!(result.api_calls, 4);
    assert_eq!(reviewer_calls.load(Ordering::SeqCst), 3);
    assert_eq!(learning.snapshot().lessons.len(), 1);
}

#[tokio::test]
async fn proposer_query_failures_are_counted_and_retried_three_times() {
    let temp = tempfile::tempdir().unwrap();
    let (mut learning, decision_id) = session(temp.path());
    let proposer_calls = Arc::new(AtomicUsize::new(0));
    let proposer_counter = proposer_calls.clone();
    let provider = LlmProvider::scripted(move |system, prompt, _effort| {
        if system.contains("LESSON_FACT_REVIEWER_V3") {
            return Ok(approval(prompt));
        }
        let call = proposer_counter.fetch_add(1, Ordering::SeqCst);
        if call < 2 {
            anyhow::bail!("temporary proposer failure {call}");
        }
        assert!(prompt.contains("retry_context"));
        assert!(prompt.contains("temporary proposer failure"));
        Ok(proposal(
            &decision_id,
            "Preserve HP before extending setup.",
        ))
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
    assert_eq!(result.api_calls, 4);
    assert_eq!(proposer_calls.load(Ordering::SeqCst), 3);
    assert_eq!(provider.recorded_requests().len(), 4);
    assert_eq!(learning.snapshot().lessons.len(), 1);
}
