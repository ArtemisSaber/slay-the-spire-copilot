use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::card_reward_pipeline::{
    plan_card_reward_with, prefer_larger_deck_response, selector_response,
};
use super::*;

fn response_with_unknown_selector_ref() -> String {
    json!({
        "schema_version": 1,
        "selected_ref": "not-an-offered-card",
        "reason": "Malformed test response.",
        "risk": "",
        "memory_ids_used": []
    })
    .to_string()
}

fn judge_response_without_schema_version(prompt: &str) -> anyhow::Result<String> {
    let valid: Value = serde_json::from_str(&prefer_larger_deck_response(prompt)?)?;
    let mut response = valid
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("judge response should be an object"))?;
    response.remove("schema_version");
    Ok(Value::Object(response).to_string())
}

#[tokio::test]
async fn selector_retries_only_selector_then_continues_to_judge() {
    let selector_attempts = Arc::new(AtomicUsize::new(0));
    let attempts = Arc::clone(&selector_attempts);
    let provider = LlmProvider::scripted(move |system_prompt, prompt, _effort| {
        if system_prompt.contains("CARD_REWARD_CANDIDATE_SELECTOR_V1") {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;
            if attempt < 3 {
                Ok(response_with_unknown_selector_ref())
            } else {
                selector_response(prompt, 1, "Corrected selector response.")
            }
        } else if system_prompt.contains("CARD_REWARD_RESULTING_STATE_JUDGE_V1") {
            prefer_larger_deck_response(prompt)
        } else {
            anyhow::bail!("unexpected generic planner call")
        }
    });

    let (planned, requests) = plan_card_reward_with(&provider, true).await;

    assert_eq!(planned.action, AutoPlayAction::Choose(1));
    assert_eq!(planned.source, DecisionSource::Llm);
    assert_eq!(selector_attempts.load(Ordering::SeqCst), 3);
    assert_eq!(requests.len(), 4);
    assert!(requests[..3].iter().all(|request| {
        request
            .system_prompt
            .contains("CARD_REWARD_CANDIDATE_SELECTOR_V1")
    }));
    assert!(
        requests[3]
            .system_prompt
            .contains("CARD_REWARD_RESULTING_STATE_JUDGE_V1")
    );
    assert!(requests[1].prompt.contains("retry_context"));
    assert!(requests[1].prompt.contains("unknown ref"));
}

#[tokio::test]
async fn judge_retries_missing_schema_without_restarting_selector() {
    let judge_attempts = Arc::new(AtomicUsize::new(0));
    let attempts = Arc::clone(&judge_attempts);
    let provider = LlmProvider::scripted(move |system_prompt, prompt, _effort| {
        if system_prompt.contains("CARD_REWARD_CANDIDATE_SELECTOR_V1") {
            selector_response(prompt, 0, "Best forced pick.")
        } else if system_prompt.contains("CARD_REWARD_RESULTING_STATE_JUDGE_V1") {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;
            if attempt < 3 {
                judge_response_without_schema_version(prompt)
            } else {
                prefer_larger_deck_response(prompt)
            }
        } else {
            anyhow::bail!("unexpected generic planner call")
        }
    });

    let (planned, requests) = plan_card_reward_with(&provider, true).await;

    assert_eq!(planned.action, AutoPlayAction::Choose(0));
    assert_eq!(planned.source, DecisionSource::Llm);
    assert_eq!(judge_attempts.load(Ordering::SeqCst), 3);
    assert_eq!(requests.len(), 4);
    assert!(
        requests[0]
            .system_prompt
            .contains("CARD_REWARD_CANDIDATE_SELECTOR_V1")
    );
    assert!(requests[1..].iter().all(|request| {
        request
            .system_prompt
            .contains("CARD_REWARD_RESULTING_STATE_JUDGE_V1")
    }));
    assert!(requests[2].prompt.contains("retry_context"));
    assert!(requests[2].prompt.contains("schema_version"));
    let first_judge: Value = serde_json::from_str(&requests[1].prompt).unwrap();
    let retried_judge: Value = serde_json::from_str(&requests[2].prompt).unwrap();
    assert_eq!(
        first_judge["resulting_states"],
        retried_judge["resulting_states"]
    );
}
