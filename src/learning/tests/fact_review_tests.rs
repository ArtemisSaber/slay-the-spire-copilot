use std::collections::HashSet;

use crate::learning::fact_review::{FactReviewVerdict, parse_fact_review};
use crate::llm::LlmProvider;
use crate::locales::Locale;

#[test]
fn fact_review_contract_accepts_an_approval_without_issues() {
    let review = parse_fact_review(
        r#"{
            "schema_version": 1,
            "verdict": "approve",
            "feedback": null,
            "issues": []
        }"#,
        &HashSet::new(),
    )
    .unwrap();

    assert_eq!(review.verdict, FactReviewVerdict::Approve);
    assert!(review.feedback.is_none());
    assert!(review.issues.is_empty());
}

#[test]
fn fact_review_contract_rejects_unknown_evidence_ids() {
    let response = r#"{
        "schema_version": 1,
        "verdict": "reject",
        "feedback": "Correct the claimed HP change before proposing another lesson.",
        "issues": [{
            "claim": "The action preserved HP.",
            "contradicting_fact": "The recorded action lost 10 HP.",
            "decision_ids": ["invented-decision"]
        }]
    }"#;

    let error =
        parse_fact_review(response, &HashSet::from(["run-a:8:2:1".to_string()])).unwrap_err();

    assert!(error.to_string().contains("unknown decision id"));
}

#[tokio::test]
async fn fact_reviewer_is_an_isolated_heavy_model_call() {
    let provider = LlmProvider::scripted(|system_prompt, _prompt, effort| {
        assert!(system_prompt.contains("LESSON_FACT_REVIEWER_V1"));
        assert_eq!(effort.as_str(), "heavy");
        Ok(r#"{"schema_version":1,"verdict":"approve","feedback":null,"issues":[]}"#.to_string())
    });

    provider
        .query_lesson_fact_review("review this draft", &Locale::load("en"))
        .await
        .unwrap();

    let requests = provider.recorded_requests();
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0]
            .system_prompt
            .contains("LESSON_FACT_REVIEWER_V1")
    );
    assert_eq!(requests[0].effort.as_str(), "heavy");
}
