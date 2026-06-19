use super::*;

#[test]
fn system_prompt_is_defined() {
    assert!(!SYSTEM_PROMPT.is_empty());
    assert!(SYSTEM_PROMPT.contains("杀戮尖塔"));
    assert!(SYSTEM_PROMPT.contains("推荐："));
    assert!(SYSTEM_PROMPT.contains("理由："));
    assert!(SYSTEM_PROMPT.contains("风险："));
    assert!(SYSTEM_PROMPT.contains("吐槽："));
}

#[test]
fn effort_from_screen_type_card_reward_is_heavy() {
    assert!(matches!(
        Effort::from_screen_type("CARD_REWARD"),
        Effort::Heavy
    ));
}

#[test]
fn effort_from_screen_type_none_is_fast() {
    assert!(matches!(Effort::from_screen_type("NONE"), Effort::Fast));
}

#[test]
fn effort_from_screen_type_other_is_medium() {
    assert!(matches!(Effort::from_screen_type("REST"), Effort::Medium));
    assert!(matches!(
        Effort::from_screen_type("UNKNOWN"),
        Effort::Medium
    ));
}

#[tokio::test]
async fn mock_provider_returns_structured_response() {
    let provider = LlmProvider::Mock;
    let result = provider.query("test prompt", Effort::Fast).await;
    assert!(result.is_ok());
    let text = result.unwrap();
    assert!(!text.is_empty());
    assert!(text.contains("推荐："));
    assert!(text.contains("理由："));
    assert!(text.contains("风险："));
    assert!(text.contains("吐槽："));
}
