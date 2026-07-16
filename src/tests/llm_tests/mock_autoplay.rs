use super::*;

#[test]
fn mock_autoplay_fallback_test_marker_returns_invalid_action() {
    let prompt = r#"{"scenario":{"test_marker":"fallback_test_marker"},"available_actions":[{"ref":"A0","kind":"play","label":"Strike"}],"rejected_attempts":["previous fail"]}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["ref"], "A99");
}

#[test]
fn mock_autoplay_retry_test_marker_no_rejections_returns_invalid_action() {
    let prompt = r#"{"scenario":{"test_marker":"retry_test_marker"},"available_actions":[{"ref":"A0","kind":"play","label":"Strike"}],"rejected_attempts":[]}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["ref"], "A99");
}

#[test]
fn mock_autoplay_prefers_card_reward_skip() {
    let prompt = r#"{"scenario":{},"available_actions":[{"ref":"A0","kind":"skip","label":"Skip"},{"ref":"A1","kind":"choose","label":"Card 0"}]}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["ref"], "A0");
}

#[test]
fn mock_autoplay_prefers_play_action() {
    let prompt = r#"{"scenario":{},"available_actions":[{"ref":"A0","kind":"end","label":"End Turn"},{"ref":"A1","kind":"play","label":"Strike"}]}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["ref"], "A1");
}

#[test]
fn mock_autoplay_first_action_fallback() {
    let prompt = r#"{"scenario":{},"available_actions":[{"ref":"A0","kind":"end","label":"End Turn"},{"ref":"A1","kind":"proceed","label":"Proceed"}]}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["ref"], "A0");
}

#[test]
fn mock_autoplay_empty_actions_returns_empty() {
    let prompt = r#"{"scenario":{},"available_actions":[]}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["schema_version"], 2);
    assert!(json["actions"].as_array().unwrap().is_empty());
}

#[test]
fn mock_autoplay_target_required_sets_target_index() {
    let prompt = r#"{"scenario":{"combat":{"monsters":[{"index":4}]}},"available_actions":[{"ref":"A0","kind":"play","label":"Strike","target_required":true}]}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["target_index"], 4);
}

#[test]
fn mock_autoplay_no_target_required_sets_null_target_index() {
    let prompt = r#"{"scenario":{},"available_actions":[{"ref":"A0","kind":"play","label":"Strike","target_required":false}]}"#;
    let response = mock_autoplay_action_response(prompt);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(json["actions"][0]["target_index"], serde_json::Value::Null);
}

#[tokio::test]
async fn mock_provider_query_autoplay_action_returns_json() {
    let provider = LlmProvider::Mock;
    let prompt =
        r#"{"scenario":{},"available_actions":[{"ref":"A0","kind":"play","label":"Strike"}]}"#;
    let result = provider
        .query_autoplay_action(prompt, Effort::Fast, test_locale())
        .await;
    assert!(result.is_ok());
    let text = result.unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["actions"][0]["ref"], "A0");
}

#[tokio::test]
async fn mock_provider_query_advice_error_path() {
    let provider = LlmProvider::Mock;
    let result = provider
        .query_advice(
            "TRIGGER_LLM_ERROR",
            Effort::Fast,
            AdviceScenario::Generic,
            test_locale(),
        )
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("mock error"));
}
