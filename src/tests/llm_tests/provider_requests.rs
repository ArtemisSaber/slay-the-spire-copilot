use super::*;

#[tokio::test]
async fn mock_provider_returns_structured_response() {
    let provider = LlmProvider::Mock;
    let result = provider
        .query_advice(
            "test prompt",
            Effort::Fast,
            AdviceScenario::Generic,
            test_locale(),
        )
        .await;
    assert!(result.is_ok());
    let text = result.unwrap();
    assert!(!text.is_empty());
    assert!(text.contains("推荐："));
    assert!(text.contains("理由："));
    assert!(text.contains("风险："));
    assert!(text.contains("吐槽："));
}

#[tokio::test]
async fn mock_provider_returns_postmortem_report() {
    let provider = LlmProvider::Mock;
    let result = provider
        .query_postmortem("deterministic summary", test_locale())
        .await;
    assert!(result.is_ok());
    let text = result.unwrap();
    assert!(text.contains("# 本局复盘"));
    assert!(text.contains("## 总览"));
    assert!(text.contains("## 下次改进"));
}

#[tokio::test]
async fn mock_provider_returns_learning_postmortem_envelope() {
    let provider = LlmProvider::Mock;
    let response = provider
        .query_learning_postmortem("critic input", test_locale())
        .await
        .unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&response).unwrap();

    assert_eq!(envelope["schema_version"], 3);
    assert!(
        envelope["report_markdown"]
            .as_str()
            .is_some_and(|report| report.starts_with('#'))
    );
    assert_eq!(envelope["result"], "lesson");
    assert!(envelope["lesson"].is_object());
    assert!(envelope["rejected_lesson_analysis"].is_null());
}

#[test]
fn chat_completion_body_can_disable_thinking() {
    let cfg = OpenAiConfig {
        model: "deepseek-v4-flash".into(),
        max_tokens: 300,
        disable_thinking: true,
    };

    let body = chat_completion_body(&cfg, "system", "user", 0.7);

    assert_eq!(body["thinking"]["type"], "disabled");
}

#[test]
fn chat_completion_body_omits_thinking_when_not_configured() {
    let cfg = OpenAiConfig {
        model: "gpt-5-nano".into(),
        max_tokens: 300,
        disable_thinking: false,
    };

    let body = chat_completion_body(&cfg, "system", "user", 0.7);

    assert!(body.get("thinking").is_none());
}

#[test]
fn anthropic_messages_body_uses_system_and_user_prompt() {
    let cfg = OpenAiConfig {
        model: "claude-sonnet-4-6".into(),
        max_tokens: 500,
        disable_thinking: false,
    };

    let body = anthropic_messages_body(&cfg, "system", "user");

    assert_eq!(body["model"], "claude-sonnet-4-6");
    assert_eq!(body["system"], "system");
    assert_eq!(body["messages"][0]["role"], "user");
    assert_eq!(body["messages"][0]["content"], "user");
    assert_eq!(body["max_tokens"], 500);
    assert!(body.get("temperature").is_none());
}

#[test]
fn anthropic_response_text_collects_text_blocks() {
    let json = serde_json::json!({
        "content": [
            {"type": "text", "text": "hello"},
            {"type": "thinking", "thinking": "..."},
            {"type": "text", "text": " world"}
        ]
    });

    let text = anthropic_response_text(&json).unwrap();

    assert_eq!(text, "hello world");
}

#[test]
fn chat_response_text_extracts_openai_compatible_content() {
    let json = serde_json::json!({
        "choices": [
            {"message": {"content": "hello"}}
        ]
    });

    let text = chat_response_text(&json).unwrap();

    assert_eq!(text, "hello");
}
