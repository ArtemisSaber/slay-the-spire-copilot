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
            "schema_version": 2,
            "verdict": "approve",
            "feedback": null,
            "claim_checks": [{
                "path": "lesson.text",
                "claim": "Use the observed safe line in comparable fights.",
                "status": "supported",
                "citations": [{
                    "source": "run_evidence",
                    "reference": decision_id,
                    "fact": "The cited decision records the observed action and outcome."
                }]
            }]
        })
        .to_string(),
        &HashSet::from([decision_id.to_string()]),
        &required_claims(),
        "Machine summary",
    )
    .unwrap();

    assert_eq!(review.verdict, FactReviewVerdict::Approve);
    assert_eq!(review.claim_checks.len(), 1);
}

#[test]
fn fact_review_contract_rejects_evidence_free_approval() {
    let error = parse_fact_review(
        r#"{
            "schema_version": 2,
            "verdict": "approve",
            "feedback": null,
            "claim_checks": []
        }"#,
        &HashSet::new(),
        &required_claims(),
        "Machine summary",
    )
    .unwrap_err();

    assert!(error.to_string().contains("every required claim"));
}

#[test]
fn fact_review_contract_accepts_rejection_for_an_unsupported_claim() {
    let review = parse_fact_review(
        r#"{
            "schema_version": 2,
            "verdict": "reject",
            "feedback": "No supplied fact supports this strategic claim.",
            "claim_checks": [{
                "path": "lesson.text",
                "claim": "Use the observed safe line in comparable fights.",
                "status": "unsupported",
                "citations": []
            }]
        }"#,
        &HashSet::new(),
        &required_claims(),
        "Machine summary",
    )
    .unwrap();

    assert_eq!(review.verdict, FactReviewVerdict::Reject);
}

#[test]
fn fact_review_contract_rejects_unknown_evidence_references() {
    let response = serde_json::json!({
        "schema_version": 2,
        "verdict": "approve",
        "feedback": null,
        "claim_checks": [{
            "path": "lesson.text",
            "claim": "Use the observed safe line in comparable fights.",
            "status": "supported",
            "citations": [{
                "source": "run_evidence",
                "reference": "invented-decision",
                "fact": "Invented support."
            }]
        }]
    })
    .to_string();

    let error = parse_fact_review(
        &response,
        &HashSet::from(["run-a:8:2:1".to_string()]),
        &required_claims(),
        "Machine summary",
    )
    .unwrap_err();

    assert!(error.to_string().contains("unknown run evidence reference"));
}

#[tokio::test]
async fn fact_reviewer_is_an_isolated_heavy_model_call_with_proof_burden() {
    let provider = LlmProvider::scripted(|system_prompt, _prompt, effort| {
        assert!(system_prompt.contains("LESSON_FACT_REVIEWER_V2"));
        assert!(system_prompt.contains("Absence of contradiction is not support"));
        assert_eq!(effort.as_str(), "heavy");
        Ok(r#"{"schema_version":2,"verdict":"reject","feedback":"Unsupported.","claim_checks":[]}"#.to_string())
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
            .contains("LESSON_FACT_REVIEWER_V2")
    );
    assert_eq!(requests[0].effort.as_str(), "heavy");
}
