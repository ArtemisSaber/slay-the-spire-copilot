use std::collections::{BTreeMap, HashSet};

use crate::learning::fact_review::{FactReviewVerdict, parse_fact_review};
use crate::llm::LlmProvider;

fn required_claims() -> BTreeMap<String, String> {
    BTreeMap::from([(
        "lesson.text".into(),
        "Use the observed safe line in comparable fights.".into(),
    )])
}

#[test]
fn fact_review_contract_accepts_approval_with_support_for_every_claim() {
    let decision_id = "run-a:8:2:1";
    let review = parse_fact_review(
        &serde_json::json!({
            "schema_version": 4,
            "verdict": "approve",
            "feedback": null,
            "checks": [{
                "path": "lesson.text",
                "status": "supported",
                "refs": [decision_id]
            }]
        })
        .to_string(),
        &HashSet::from([decision_id.to_string()]),
        &required_claims(),
    )
    .unwrap();

    assert_eq!(review.verdict, FactReviewVerdict::Approve);
    assert_eq!(review.checks.len(), 1);
}

#[test]
fn fact_review_contract_accepts_a_grounded_plausible_counterfactual() {
    let decision_id = "run-a:8:2:1";
    let review = parse_fact_review(
        &serde_json::json!({
            "schema_version": 4,
            "verdict": "approve",
            "feedback": null,
            "checks": [{
                "path": "lesson.text",
                "status": "plausible",
                "refs": [decision_id]
            }]
        })
        .to_string(),
        &HashSet::from([decision_id.to_string()]),
        &required_claims(),
    )
    .unwrap();

    assert_eq!(review.verdict, FactReviewVerdict::Approve);
}

#[test]
fn fact_review_contract_requires_observed_chains_to_be_supported() {
    let decision_id = "run-a:8:2:1";
    let claims = BTreeMap::from([(
        "lesson.evidence[0].observed_chain".into(),
        "The player lost 12 HP after the cited decision.".into(),
    )]);
    let error = parse_fact_review(
        &serde_json::json!({
            "schema_version": 4,
            "verdict": "approve",
            "feedback": null,
            "checks": [{
                "path": "lesson.evidence[0].observed_chain",
                "status": "plausible",
                "refs": [decision_id]
            }]
        })
        .to_string(),
        &HashSet::from([decision_id.to_string()]),
        &claims,
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("observed chains require support")
    );
}

#[test]
fn fact_review_contract_rejects_ungrounded_plausibility() {
    let error = parse_fact_review(
        r#"{
            "schema_version": 4,
            "verdict": "approve",
            "feedback": null,
            "checks": [{
                "path": "lesson.text",
                "status": "plausible",
                "refs": []
            }]
        }"#,
        &HashSet::new(),
        &required_claims(),
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("grounded statuses require at least one reference")
    );
}

#[test]
fn fact_review_contract_cannot_reject_a_grounded_hypothesis_only_for_being_plausible() {
    let decision_id = "run-a:8:2:1";
    let error = parse_fact_review(
        &serde_json::json!({
            "schema_version": 4,
            "verdict": "reject",
            "feedback": "The alternative was not directly observed.",
            "checks": [{
                "path": "lesson.text",
                "status": "plausible",
                "refs": [decision_id]
            }]
        })
        .to_string(),
        &HashSet::from([decision_id.to_string()]),
        &required_claims(),
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("unsupported or contradicted claim")
    );
}

#[test]
fn fact_review_contract_rejects_evidence_free_approval() {
    let error = parse_fact_review(
        r#"{
            "schema_version": 4,
            "verdict": "approve",
            "feedback": null,
            "checks": []
        }"#,
        &HashSet::new(),
        &required_claims(),
    )
    .unwrap_err();

    assert!(error.to_string().contains("every required claim"));
}

#[test]
fn fact_review_contract_accepts_rejection_for_an_unsupported_claim() {
    let review = parse_fact_review(
        r#"{
            "schema_version": 4,
            "verdict": "reject",
            "feedback": "No supplied fact supports this strategic claim.",
            "checks": [{
                "path": "lesson.text",
                "status": "unsupported",
                "refs": ["run-a:8:2:1"]
            }]
        }"#,
        &HashSet::from(["run-a:8:2:1".to_string()]),
        &required_claims(),
    )
    .unwrap();

    assert_eq!(review.verdict, FactReviewVerdict::Reject);
}

#[test]
fn fact_review_contract_rejects_unknown_evidence_references() {
    let response = serde_json::json!({
        "schema_version": 4,
        "verdict": "approve",
        "feedback": null,
        "checks": [{
            "path": "lesson.text",
            "status": "supported",
            "refs": ["invented-decision"]
        }]
    })
    .to_string();

    let error = parse_fact_review(
        &response,
        &HashSet::from(["run-a:8:2:1".to_string()]),
        &required_claims(),
    )
    .unwrap_err();

    assert!(error.to_string().contains("unknown reference"));
}

#[tokio::test]
async fn fact_reviewer_is_an_isolated_heavy_reasoning_call() {
    let provider = LlmProvider::scripted(|system_prompt, _prompt, effort| {
        assert!(system_prompt.contains("LESSON_FACT_REVIEWER_V4"));
        assert!(system_prompt.contains("established Slay the Spire knowledge"));
        assert!(system_prompt.contains("ordinary logic, arithmetic, and strategic reasoning"));
        assert!(system_prompt.contains("need not appear verbatim"));
        assert!(system_prompt.contains("plausible"));
        assert!(!system_prompt.contains("Never use unstated game knowledge"));
        assert!(system_prompt.contains("path, status, and refs"));
        assert_eq!(effort.as_str(), "heavy");
        Ok(
            r#"{"schema_version":4,"verdict":"reject","feedback":"Unsupported.","checks":[]}"#
                .to_string(),
        )
    });

    provider
        .query_lesson_fact_review("review this draft")
        .await
        .unwrap();

    let requests = provider.recorded_requests();
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0]
            .system_prompt
            .contains("LESSON_FACT_REVIEWER_V4")
    );
    assert_eq!(requests[0].effort.as_str(), "heavy");
}
