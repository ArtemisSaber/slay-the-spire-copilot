use super::*;
use crate::llm::{AdviceScenario, Effort, LlmProvider};
use crate::state::ScreenType;
use crate::test_utils::test_locale;

#[test]
fn parse_advice_response_all_fields() {
    let raw = "推荐：武装\n理由：攻击牌占比过高(10/16)，需要技能牌来平衡攻防节奏。\n风险：武装前期抽到且手牌无高价值目标时会卡手。\n吐槽：这卡组攻击力爆表但像个莽夫！";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "武装");
    assert_eq!(
        fields.reason,
        "攻击牌占比过高(10/16)，需要技能牌来平衡攻防节奏。"
    );
    assert_eq!(fields.risk, "武装前期抽到且手牌无高价值目标时会卡手。");
    assert_eq!(fields.commentary, "这卡组攻击力爆表但像个莽夫！");
}

#[test]
fn parse_advice_response_partial_fields() {
    let raw = "推荐：跳过\n理由：都不好";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "跳过");
    assert_eq!(fields.reason, "都不好");
    assert_eq!(fields.risk, "");
    assert_eq!(fields.commentary, "");
}

#[test]
fn parse_advice_response_multiline_reason() {
    let raw = "推荐：武装\n理由：攻击牌占比过高\n需要更多技能牌\n平衡攻防节奏\n风险：前期卡手";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "武装");
    assert_eq!(
        fields.reason,
        "攻击牌占比过高\n需要更多技能牌\n平衡攻防节奏"
    );
    assert_eq!(fields.risk, "前期卡手");
    assert_eq!(fields.commentary, "");
}

#[test]
fn parse_advice_response_multiline_commentary() {
    let raw = "推荐：跳过\n理由：没有好牌\n风险：错过发育机会\n吐槽：哈哈\n选牌也能这么背";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.commentary, "哈哈\n选牌也能这么背");
}

#[test]
fn parse_advice_response_empty() {
    let fields = parse_advice_response("", test_locale());
    assert_eq!(fields.recommendation, "");
    assert_eq!(fields.reason, "");
    assert_eq!(fields.risk, "");
    assert_eq!(fields.commentary, "");
}

#[test]
fn parse_advice_response_content_before_first_field_is_ignored() {
    let raw = "形势不错。  角色：铁甲战士  层数：5\n推荐：武装\n理由：好";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "武装");
    assert_eq!(fields.reason, "好");
}

#[test]
fn parse_advice_response_blank_lines_between_fields() {
    let raw = "推荐：武装\n\n理由：好\n\n\n风险：弱\n\n吐槽：烂";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "武装");
    assert_eq!(fields.reason, "好");
    assert_eq!(fields.risk, "弱");
    assert_eq!(fields.commentary, "烂");
}

#[test]
fn parse_advice_response_only_commentary() {
    let raw = "吐槽：这个卡组太极端了";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "");
    assert_eq!(fields.reason, "");
    assert_eq!(fields.risk, "");
    assert_eq!(fields.commentary, "这个卡组太极端了");
}

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

#[test]
fn parse_advice_response_real_example_card_reward() {
    let raw = "推荐：武装\n理由：攻击牌占比过高(10/16)，需要技能牌来平衡攻防节奏。武装的低费和升级能力能提升整副卡组的质量。\n风险：武装前期抽到且手牌无高价值目标时会卡手。\n吐槽：这卡组攻击力爆表但像个莽夫！学点生存技巧吧，别光想着打打打！";

    let fields = parse_advice_response(raw, test_locale());

    assert!(!fields.recommendation.is_empty());
    assert!(!fields.reason.is_empty());
    assert!(!fields.risk.is_empty());
    assert!(!fields.commentary.is_empty());
}

#[test]
fn parse_advice_response_real_example_rest() {
    let raw = "推荐：休息\n理由：血量极低(12/75)，下一场战斗可能遇到精英，必须保证生存。\n风险：错过锻造机会，卡组强度提升推迟。\n吐槽：活着才有输出！别贪了，先回血保命吧。";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "休息");
    assert!(fields.reason.contains("血量极低"));
    assert!(fields.risk.contains("锻造"));
    assert!(fields.commentary.contains("活着"));
}

#[test]
fn parse_advice_response_field_names_with_punctuation() {
    let raw = "推荐：跳过。\n理由：都不好。\n风险：无。\n吐槽：。";

    let fields = parse_advice_response(raw, test_locale());

    assert_eq!(fields.recommendation, "跳过。");
    assert_eq!(fields.reason, "都不好。");
    assert_eq!(fields.risk, "无。");
    assert_eq!(fields.commentary, "。");
}

#[test]
fn write_overlay_json_to_creates_file_and_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let overlay_path = dir.path().join("nested").join("overlay.json");

    let output = OverlayOutput {
        schema_version: 1,
        status: "ok".to_string(),
        overlay_visibility: true,
        advice: AdviceFields {
            recommendation: "测试".to_string(),
            reason: "".to_string(),
            risk: "".to_string(),
            commentary: "".to_string(),
        },
        screen_type: None,
        scenario: "generic".to_string(),
        in_combat: false,
        state_hash: "test".to_string(),
        floor: None,
        character: None,
        timestamp_ms: 0,
    };

    write_overlay_json_to(&overlay_path, &output);

    assert!(
        overlay_path.exists(),
        "overlay.json should be created in nested dir"
    );
    let content = std::fs::read_to_string(&overlay_path).unwrap();
    assert!(content.contains("\"recommendation\": \"测试\""));
    assert!(content.contains("\"overlay_visibility\": true"));
    assert!(content.contains("\"schema_version\": 1"));
}

// Existing tests (kept)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cache_returns_cached_value() {
    let provider = LlmProvider::Mock;
    let mut cache = AdviceCache::new();

    let hash = "abc123";
    let prompt = "test prompt";

    let first = cache
        .get_or_compute(
            hash,
            prompt,
            Effort::Fast,
            AdviceScenario::Generic,
            &provider,
            test_locale(),
        )
        .await;
    let second = cache
        .get_or_compute(
            hash,
            prompt,
            Effort::Fast,
            AdviceScenario::Generic,
            &provider,
            test_locale(),
        )
        .await;

    assert_eq!(first, second);
}

#[test]
fn write_advice_output_path_behavior() {
    let cwd_file = crate::logging::advice_output_dir()
        .join("output")
        .join("advice.txt");
    let binary_file = crate::logging::project_root()
        .join("output")
        .join("advice.txt");

    let _ = std::fs::remove_file(&cwd_file);
    let _ = std::fs::remove_file(&binary_file);
    let _ = std::fs::remove_dir_all(crate::logging::advice_output_dir().join("output"));

    let cache = AdviceCache::new();

    cache.write_advice("first");
    assert!(
        cwd_file.exists(),
        "should create output/ dir and write to cwd"
    );

    if cwd_file != binary_file {
        assert!(
            !binary_file.exists(),
            "should not write to binary's output/ when cwd differs"
        );
    }

    cache.write_advice("second");
    let content = std::fs::read_to_string(&cwd_file).unwrap();
    assert!(
        !content.contains("first"),
        "latest write should replace previous"
    );
    assert!(content.contains("second"));

    let _ = std::fs::remove_file(&cwd_file);
}

#[tokio::test]
async fn get_or_compute_llm_error_fallback() {
    let provider = LlmProvider::Mock;
    let mut cache = AdviceCache::new();
    let result = cache
        .get_or_compute(
            "hash1",
            "TRIGGER_LLM_ERROR",
            Effort::Fast,
            AdviceScenario::Generic,
            &provider,
            test_locale(),
        )
        .await;
    assert_eq!(result, "LLM 调用失败，请检查配置。");
}

#[test]
fn timestamp_ms_returns_positive() {
    let ts = timestamp_ms();
    assert!(ts > 1_700_000_000_000);
}

#[test]
fn timestamp_ms_is_monotonic() {
    let ts1 = timestamp_ms();
    let ts2 = timestamp_ms();
    assert!(ts2 >= ts1);
}

#[test]
fn atomic_write_json_creates_nested_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("a").join("b").join("test.json");
    atomic_write_json(&nested, r#"{"key":"value"}"#);
    assert!(nested.exists());
    assert_eq!(
        std::fs::read_to_string(&nested).unwrap(),
        r#"{"key":"value"}"#
    );
}

#[test]
fn atomic_write_json_replaces_existing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.json");
    std::fs::write(&path, r#"{"old":true}"#).unwrap();
    atomic_write_json(&path, r#"{"new":true}"#);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), r#"{"new":true}"#);
    assert!(!path.with_extension("json.tmp").exists());
}

#[test]
fn atomic_write_json_root_level() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("root.json");
    atomic_write_json(&path, r#"{"a":1}"#);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), r#"{"a":1}"#);
}

#[test]
fn write_overlay_loading_produces_valid_json() {
    let dir = tempfile::tempdir().unwrap();
    let overlay_path = dir.path().join("output").join("overlay.json");
    std::fs::create_dir_all(dir.path().join("output")).unwrap();

    let output = OverlayOutput {
        schema_version: 1,
        status: "loading".to_string(),
        overlay_visibility: true,
        advice: AdviceFields::default(),
        screen_type: Some(ScreenType::CardReward),
        scenario: "card_reward".to_string(),
        in_combat: false,
        state_hash: "abc123".to_string(),
        floor: Some(5),
        character: Some("IRONCLAD".to_string()),
        timestamp_ms: timestamp_ms(),
    };
    write_overlay_json_to(&overlay_path, &output);

    let content = std::fs::read_to_string(&overlay_path).unwrap();
    assert!(content.contains("\"status\": \"loading\""));
    assert!(content.contains("\"overlay_visibility\": true"));

    let _ = std::fs::remove_dir_all(dir.path().join("output"));
}

#[test]
fn write_overlay_ready_produces_valid_json() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("output")).unwrap();
    let overlay_path = dir.path().join("output").join("overlay.json");

    let output = OverlayOutput {
        schema_version: 1,
        status: "ok".to_string(),
        overlay_visibility: true,
        advice: AdviceFields {
            recommendation: "武装".into(),
            reason: "好".into(),
            risk: "".into(),
            commentary: "".into(),
        },
        screen_type: None,
        scenario: "generic".to_string(),
        in_combat: false,
        state_hash: "xyz".to_string(),
        floor: None,
        character: None,
        timestamp_ms: timestamp_ms(),
    };
    write_overlay_json_to(&overlay_path, &output);

    let content = std::fs::read_to_string(&overlay_path).unwrap();
    assert!(content.contains("\"status\": \"ok\""));
    assert!(content.contains("\"overlay_visibility\": true"));
    assert!(content.contains("\"recommendation\": \"武装\""));

    let _ = std::fs::remove_dir_all(dir.path().join("output"));
}

#[test]
fn atomic_write_json_tmp_is_cleaned_up() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cleanup.json");
    atomic_write_json(&path, r#"{"ok":true}"#);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), r#"{"ok":true}"#);
    assert!(!path.with_extension("json.tmp").exists());
}

#[test]
fn cancel_hide_timer_when_none_is_noop() {
    let mut cache = AdviceCache::new();
    cache.cancel_hide_timer();
    assert!(cache.hide_timer.is_none());
}
