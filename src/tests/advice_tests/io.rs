use super::*;

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
