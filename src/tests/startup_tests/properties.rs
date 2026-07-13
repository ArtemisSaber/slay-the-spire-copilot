use super::*;

#[test]
fn extract_property_value_skips_commented_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "# command=hidden\ncommand=real\n").unwrap();
    assert_eq!(
        extract_property_value(&path, "command"),
        Some("real".to_string())
    );
}

#[test]
fn extract_property_value_handles_indented_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "    command=indented\n").unwrap();
    assert_eq!(
        extract_property_value(&path, "command"),
        Some("indented".to_string())
    );
}

#[test]
fn extract_property_value_missing_key_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "other=value\n").unwrap();
    assert_eq!(extract_property_value(&path, "command"), None);
}

#[test]
fn extract_property_value_run_at_game_start_variants() {
    for (value, expected) in [
        ("1", true),
        ("true", true),
        ("TRUE", true),
        ("yes", true),
        ("Yes", true),
        ("on", true),
        ("ON", true),
        ("0", false),
        ("false", false),
        ("no", false),
        ("off", false),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.properties");
        std::fs::write(&path, format!("runAtGameStart={value}\n")).unwrap();
        assert_eq!(
            run_at_game_start_enabled(&path),
            expected,
            "runAtGameStart={value} should be {expected}"
        );
    }
}

#[test]
fn run_at_game_start_enabled_absent_returns_false() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "command=/bin/copilot\n").unwrap();
    assert!(!run_at_game_start_enabled(&path));
}
