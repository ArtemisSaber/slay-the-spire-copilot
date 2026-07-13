use super::*;

#[test]
fn advice_fields_defaults() {
    let fields = AdviceFields::default();
    assert_eq!(fields.recommendation, "");
    assert_eq!(fields.reason, "");
    assert_eq!(fields.risk, "");
    assert_eq!(fields.commentary, "");
}

#[test]
fn overlay_output_serialization_structure() {
    let output = OverlayOutput {
        schema_version: 1,
        status: "ok".to_string(),
        overlay_visibility: true,
        advice: AdviceFields {
            recommendation: "武装".to_string(),
            reason: "好".to_string(),
            risk: "卡手".to_string(),
            commentary: "还行".to_string(),
        },
        screen_type: Some(ScreenType::CardReward),
        scenario: "card_reward".to_string(),
        in_combat: false,
        state_hash: "deadbeef".to_string(),
        floor: Some(12),
        character: Some("IRONCLAD".to_string()),
        timestamp_ms: 1711234567890,
    };

    let json = serde_json::to_string_pretty(&output).unwrap();

    assert!(json.contains("\"schema_version\": 1"));
    assert!(json.contains("\"status\": \"ok\""));
    assert!(json.contains("\"overlay_visibility\": true"));
    assert!(json.contains("\"advice\": {"));
    assert!(json.contains("\"recommendation\": \"武装\""));
    assert!(json.contains("\"reason\": \"好\""));
    assert!(json.contains("\"risk\": \"卡手\""));
    assert!(json.contains("\"commentary\": \"还行\""));
    assert!(json.contains("\"screen_type\": \"CARD_REWARD\""));
    assert!(json.contains("\"scenario\": \"card_reward\""));
    assert!(json.contains("\"in_combat\": false"));
    assert!(json.contains("\"state_hash\": \"deadbeef\""));
    assert!(json.contains("\"floor\": 12"));
    assert!(json.contains("\"character\": \"IRONCLAD\""));
    assert!(json.contains("\"timestamp_ms\": 1711234567890"));
}

#[test]
fn overlay_output_loading_state_empty_advice() {
    let output = OverlayOutput {
        schema_version: 1,
        status: "loading".to_string(),
        overlay_visibility: true,
        advice: AdviceFields::default(),
        screen_type: Some(ScreenType::CardReward),
        scenario: "card_reward".to_string(),
        in_combat: false,
        state_hash: "abc".to_string(),
        floor: None,
        character: None,
        timestamp_ms: 0,
    };

    let json = serde_json::to_string_pretty(&output).unwrap();

    assert!(json.contains("\"status\": \"loading\""));
    assert!(json.contains("\"overlay_visibility\": true"));
    assert!(json.contains("\"recommendation\": \"\""));
    assert!(json.contains("\"reason\": \"\""));
    assert!(json.contains("\"risk\": \"\""));
    assert!(json.contains("\"commentary\": \"\""));
}

#[test]
fn overlay_output_error_state() {
    let output = OverlayOutput {
        schema_version: 1,
        status: "error".to_string(),
        overlay_visibility: true,
        advice: AdviceFields {
            recommendation: "LLM 调用失败".to_string(),
            reason: "请检查配置。".to_string(),
            risk: "".to_string(),
            commentary: "".to_string(),
        },
        screen_type: None,
        scenario: "generic".to_string(),
        in_combat: false,
        state_hash: "".to_string(),
        floor: None,
        character: None,
        timestamp_ms: 0,
    };

    let json = serde_json::to_string_pretty(&output).unwrap();

    assert!(json.contains("\"status\": \"error\""));
    assert!(json.contains("\"recommendation\": \"LLM 调用失败\""));
}

#[test]
fn overlay_output_hidden_state() {
    let output = OverlayOutput {
        schema_version: 1,
        status: "ok".to_string(),
        overlay_visibility: false,
        advice: AdviceFields {
            recommendation: "武装".to_string(),
            reason: "好牌".to_string(),
            risk: "".to_string(),
            commentary: "".to_string(),
        },
        screen_type: None,
        scenario: "card_reward".to_string(),
        in_combat: false,
        state_hash: "xyz".to_string(),
        floor: None,
        character: None,
        timestamp_ms: 0,
    };

    let json = serde_json::to_string_pretty(&output).unwrap();

    assert!(json.contains("\"status\": \"ok\""));
    assert!(json.contains("\"overlay_visibility\": false"));
    assert!(json.contains("\"recommendation\": \"武装\""));
}

#[test]
fn overlay_metadata_construction() {
    let metadata = OverlayMetadata {
        screen_type: Some(ScreenType::CardReward),
        scenario: "card_reward".to_string(),
        in_combat: false,
        state_hash: "hash123".to_string(),
        floor: Some(7),
        character: Some("IRONCLAD".to_string()),
    };

    assert_eq!(
        metadata.screen_type.as_ref().map(|st| st.as_str()),
        Some("CARD_REWARD")
    );
    assert_eq!(metadata.scenario, "card_reward");
    assert!(!metadata.in_combat);
    assert_eq!(metadata.state_hash, "hash123");
    assert_eq!(metadata.floor, Some(7));
    assert_eq!(metadata.character.as_deref(), Some("IRONCLAD"));
}
