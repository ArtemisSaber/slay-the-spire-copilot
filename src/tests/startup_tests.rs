use super::*;
use std::env;
use std::fs;
use std::path::PathBuf;

fn temp_config(content: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    fs::write(&path, content).unwrap();
    (dir, path)
}

#[test]
fn communication_mod_cjk_config_dir_is_preferred() {
    assert_eq!(COMMUNICATION_MOD_CONFIG_DIRS[0], "CommunicationModCJK");
    assert!(COMMUNICATION_MOD_CONFIG_DIRS.contains(&"CommunicationMod"));
}

#[test]
fn gameplay_settings_language_is_read_from_sts_preferences() {
    let (_dir, settings) = temp_config(
        r#"{
          "LANGUAGE": "ZHS",
          "Fast Mode": "true"
        }"#,
    );

    assert_eq!(
        read_gameplay_settings_language(&settings),
        Some("ZHS".to_string())
    );
}

#[test]
fn steam_root_builds_windows_language_paths() {
    let mut paths = Vec::new();
    push_slay_the_spire_language_path_for_steam_root(
        &mut paths,
        &PathBuf::from(r"C:\Program Files (x86)\Steam"),
    );

    assert_eq!(
        paths[0],
        PathBuf::from(r"C:\Program Files (x86)\Steam")
            .join("steamapps")
            .join("common")
            .join("SlayTheSpire")
            .join("preferences")
            .join("STSGameplaySettings")
    );
}

#[test]
fn steam_root_builds_windows_appmanifest_paths() {
    let mut paths = Vec::new();
    push_steam_appmanifest_path_for_steam_root(
        &mut paths,
        &PathBuf::from(r"C:\Program Files (x86)\Steam"),
    );

    assert_eq!(
        paths[0],
        PathBuf::from(r"C:\Program Files (x86)\Steam")
            .join("steamapps")
            .join("appmanifest_646570.acf")
    );
}

#[test]
fn cjk_languages_require_communication_mod_cjk() {
    for language in [
        "ZHS", "ZHT", "schinese", "tchinese", "Japanese", "koreana", "zh_CN", "ja-JP", "ko_KR",
    ] {
        assert!(
            language_needs_cjk_mod(language),
            "{language} should require Communication Mod CJK"
        );
    }
}

#[test]
fn ascii_compatible_languages_do_not_require_communication_mod_cjk() {
    for language in ["ENG", "english", "french", "deu", "spanish", "rus"] {
        assert!(
            !language_needs_cjk_mod(language),
            "{language} should not require Communication Mod CJK"
        );
    }
}

#[test]
fn original_communication_mod_config_requires_cjk_prompt_for_cjk_language() {
    let path = PathBuf::from("/tmp/ModTheSpire/CommunicationMod/config.properties");
    assert!(cjk_mod_required_for_language(&path, Some("ZHS")));
}

#[test]
fn cjk_communication_mod_config_does_not_require_cjk_prompt() {
    let path = PathBuf::from("/tmp/ModTheSpire/CommunicationModCJK/config.properties");
    assert!(!cjk_mod_required_for_language(&path, Some("ZHS")));
}

#[test]
fn command_parser_preserves_unquoted_windows_exe_paths_with_spaces() {
    assert_eq!(
        parse_command_value(r"C:\Users\Howard Lee\bin\slay-the-spire-copilot.exe"),
        Some(r"C:\Users\Howard Lee\bin\slay-the-spire-copilot.exe".to_string())
    );
}

#[test]
fn command_parser_preserves_unquoted_windows_exe_paths_with_args() {
    assert_eq!(
        parse_command_value(
            r"C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe --stdin-test"
        ),
        Some(r"C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe".to_string())
    );
}

#[test]
fn command_parser_handles_quoted_windows_paths() {
    assert_eq!(
        parse_command_value(
            r#""C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe" --stdin-test"#
        ),
        Some(r"C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe".to_string())
    );
}

#[test]
fn write_command_quotes_paths_with_spaces() {
    let (_dir, config) = temp_config("command=\n");
    let command = r"C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe";

    assert!(write_command_to_config(&config, command));

    let updated = fs::read_to_string(&config).unwrap();
    assert!(updated.contains(&format!("command=\"{command}\"")));
}

#[test]
fn correct_config_passes() {
    let current = env::current_exe().unwrap();
    let (_dir, config) = temp_config(&format!("command={}\n", current.display()));
    let paths = vec![config];
    assert!(check_config_matches_current_exe(&paths));
}

#[test]
fn wrong_config_fails() {
    let (_dir, config) = temp_config("command=/usr/bin/other-bot\n");
    let paths = vec![config];
    assert!(!check_config_matches_current_exe(&paths));
}

#[test]
fn empty_command_auto_fixes() {
    let current = env::current_exe().unwrap().display().to_string();
    let (_dir, config) = temp_config("command=\n");
    assert!(write_command_to_config(&config, &current));
    let updated = fs::read_to_string(&config).unwrap();
    assert!(updated.contains(&format!("command={current}")));
}

#[test]
fn no_config_at_all() {
    let paths = vec![PathBuf::from("/tmp/nonexistent-config-12345.properties")];
    assert!(!check_config_matches_current_exe(&paths));
}

#[test]
fn setup_message_contains_key_info() {
    let mut buf = Vec::new();
    show_setup_message_to(
        &mut buf,
        "/test/config/path",
        "/usr/bin/slay-the-spire-copilot",
    );
    let output = String::from_utf8(buf).unwrap();

    assert!(output.contains("尚未配置"));
    assert!(output.contains("ModTheSpire"));
    assert!(output.contains("CommunicationMod"));
    assert!(output.contains("Communication Mod CJK"));
    assert!(output.contains("steamcommunity.com/sharedfiles/filedetails/?id=3748153752"));
    assert!(output.contains("github.com/ArtemisSaber/CommunicationMod/releases"));
    assert!(output.contains("command="));
    assert!(output.contains("runAtGameStart=true"));
    assert!(output.contains("/test/config/path"));
    assert!(output.contains("/usr/bin/slay-the-spire-copilot"));
}
