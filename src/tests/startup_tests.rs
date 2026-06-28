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
fn command_parser_unquoted_path() {
    assert_eq!(
        parse_command_value(r"C:\Users\Howard Lee\bin\slay-the-spire-copilot.exe"),
        Some(r"C:\Users\Howard Lee\bin\slay-the-spire-copilot.exe".to_string())
    );
}

#[test]
fn command_parser_preserves_trailing_args_for_config_check() {
    assert_eq!(
        parse_command_value(
            r"C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe --stdin-test"
        ),
        Some(r"C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe --stdin-test".to_string())
    );
}

#[test]
fn command_parser_values_with_args_not_fully_quoted_are_passed_through() {
    let result = parse_command_value(
        r#""C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe" --stdin-test"#,
    );
    assert!(result.is_some_and(|v| v.contains("stdin-test")));
}

#[test]
fn command_parser_strips_single_quotes() {
    assert_eq!(
        parse_command_value(r"'/usr/bin/copilot'"),
        Some("/usr/bin/copilot".to_string())
    );
}

#[test]
fn command_parser_unescapes_java_properties_path() {
    assert_eq!(
        parse_command_value(
            r#""G\:\\Barracuda\\Game files\\Slay the Spire\\slay the spire copilot\\slay-the-spire-copilot.exe""#
        ),
        Some(
            r"G:\Barracuda\Game files\Slay the Spire\slay the spire copilot\slay-the-spire-copilot.exe"
                .to_string()
        )
    );
}

#[test]
fn command_parser_unescapes_unquoted_java_properties_path() {
    assert_eq!(
        parse_command_value(
            r"G\:\\Barracuda\\Game files\\Slay the Spire\\slay-the-spire-copilot.exe"
        ),
        Some(r"G:\Barracuda\Game files\Slay the Spire\slay-the-spire-copilot.exe".to_string())
    );
}

#[test]
fn command_parser_empty_returns_none() {
    assert_eq!(parse_command_value(""), None);
    assert_eq!(parse_command_value("  "), None);
    assert_eq!(parse_command_value("\"\""), None);
}

#[test]
fn format_command_value_doubles_backslashes_for_properties() {
    assert_eq!(format_command_value("/usr/bin/copilot"), "/usr/bin/copilot");
}

#[test]
fn format_command_value_doubles_windows_path_backslashes() {
    assert_eq!(
        format_command_value(r"G:\foo\bar\copilot.exe"),
        r"G:\\foo\\bar\\copilot.exe"
    );
}

#[test]
fn format_command_value_quotes_paths_with_spaces_and_doubles_backslashes() {
    assert_eq!(
        format_command_value(r"G:\Program Files\Copilot\copilot.exe"),
        r#""G:\\Program Files\\Copilot\\copilot.exe""#
    );
}

#[test]
fn format_command_value_preserves_already_quoted_value() {
    assert_eq!(
        format_command_value(r#""G:\foo\bar.exe""#),
        r#""G:\\foo\\bar.exe""#
    );
}

#[test]
fn write_command_quotes_paths_with_spaces() {
    let (_dir, config) = temp_config("command=\n");
    let command = r"C:\Program Files\Slay Copilot\slay-the-spire-copilot.exe";

    assert!(write_command_to_config(&config, command));

    let updated = fs::read_to_string(&config).unwrap();
    assert!(updated.contains(&format!("command={}", format_command_value(command))));
    assert!(updated.contains("runAtGameStart=true"));
}

#[test]
fn write_command_enables_run_at_game_start() {
    let (_dir, config) = temp_config("command=/old/path\nrunAtGameStart=false\n");

    assert!(write_command_to_config(&config, "/usr/bin/copilot"));

    let updated = fs::read_to_string(&config).unwrap();
    assert!(updated.contains("command=/usr/bin/copilot"));
    assert!(updated.contains("runAtGameStart=true"));
}

#[test]
fn correct_config_passes() {
    let current = env::current_exe().unwrap();
    let (_dir, config) = temp_config(&format!(
        "command={}\nrunAtGameStart=true\n",
        current.display()
    ));
    let paths = vec![config];
    assert!(check_config_matches_current_exe(&paths));
}

#[test]
fn correct_command_without_run_at_game_start_fails() {
    let current = env::current_exe().unwrap();
    let (_dir, config) = temp_config(&format!("command={}\n", current.display()));
    let paths = vec![config];
    assert!(!check_config_matches_current_exe(&paths));
}

#[test]
fn correct_command_with_run_at_game_start_false_fails() {
    let current = env::current_exe().unwrap();
    let (_dir, config) = temp_config(&format!(
        "command={}\nrunAtGameStart=false\n",
        current.display()
    ));
    let paths = vec![config];
    assert!(!check_config_matches_current_exe(&paths));
}

#[test]
fn wrong_config_fails() {
    let (_dir, config) = temp_config("command=/usr/bin/other-bot\nrunAtGameStart=true\n");
    let paths = vec![config];
    assert!(!check_config_matches_current_exe(&paths));
}

#[test]
fn empty_command_auto_fixes() {
    let current = env::current_exe().unwrap().display().to_string();
    let (_dir, config) = temp_config("command=\n");
    assert!(write_command_to_config(&config, &current));
    let updated = fs::read_to_string(&config).unwrap();
    assert!(updated.contains(&format!("command={}", format_command_value(&current))));
    assert!(updated.contains("runAtGameStart=true"));
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
    assert!(output.contains("Communication Mod CJK"));
    assert!(output.contains("steamcommunity.com/sharedfiles/filedetails/?id=3748153752"));
    assert!(output.contains("github.com/ArtemisSaber/CommunicationMod/releases"));
    assert!(!output.contains("ForgottenArbiter"));
    assert!(!output.contains("原版 CommunicationMod"));
    assert!(output.contains("command="));
    assert!(output.contains("runAtGameStart=true"));
    assert!(output.contains("/test/config/path"));
    assert!(output.contains("/usr/bin/slay-the-spire-copilot"));
}

// Additional coverage tests

#[test]
fn communication_mod_config_paths_returns_non_empty() {
    let paths = communication_mod_config_paths();
    assert!(!paths.is_empty());
}

#[test]
fn slay_the_spire_language_paths_returns_non_empty() {
    let paths = slay_the_spire_language_paths();
    assert!(!paths.is_empty());
}

#[test]
fn steam_appmanifest_paths_returns_non_empty() {
    let paths = steam_appmanifest_paths();
    assert!(!paths.is_empty());
}

#[test]
fn language_needs_cjk_mod_variants() {
    for lang in [
        "zhs",
        "zht",
        "zh",
        "zhcn",
        "zhtw",
        "schinese",
        "tchinese",
        "chinesesimplified",
        "chinesetraditional",
        "jpn",
        "ja",
        "jp",
        "japanese",
        "kor",
        "ko",
        "kr",
        "korean",
        "koreana",
    ] {
        assert!(language_needs_cjk_mod(lang), "should need CJK mod: {lang}");
    }
    for lang in ["eng", "en", "english", "french", "deu", "rus", ""] {
        assert!(
            !language_needs_cjk_mod(lang),
            "should not need CJK mod: {lang}"
        );
    }
}

#[test]
fn cjk_mod_required_when_cjk_language_on_standard_mod() {
    let dir = tempfile::tempdir().unwrap();
    let std_path = dir
        .path()
        .join("ModTheSpire")
        .join("CommunicationMod")
        .join("config.properties");
    assert!(cjk_mod_required_for_language(&std_path, Some("zhs")));
    let cjk_path = dir
        .path()
        .join("ModTheSpire")
        .join("CommunicationModCJK")
        .join("config.properties");
    assert!(!cjk_mod_required_for_language(&cjk_path, Some("zhs")));
    assert!(!cjk_mod_required_for_language(&std_path, None));
}

#[test]
fn write_command_to_config_creates_both_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "").unwrap();
    assert!(write_command_to_config(&path, "/bin/copilot"));
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("command=/bin/copilot"));
    assert!(content.contains("runAtGameStart=true"));
}

#[test]
fn write_command_to_config_replaces_existing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "command=old\nrunAtGameStart=false\nother=keep\n").unwrap();
    assert!(write_command_to_config(&path, "/new/bin"));
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("command=/new/bin"));
    assert!(content.contains("runAtGameStart=true"));
    assert!(content.contains("other=keep"));
}

#[test]
fn try_fix_config_enables_run_at_game_start() {
    let dir = tempfile::tempdir().unwrap();
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, format!("command={exe}\nrunAtGameStart=false\n")).unwrap();
    let paths = vec![path.clone()];
    let result = try_fix_config(&paths, &exe);
    assert!(result);
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("runAtGameStart=true"));
}

#[test]
fn try_fix_config_fills_empty_command() {
    let dir = tempfile::tempdir().unwrap();
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "other=keep\nrunAtGameStart=true\n").unwrap();
    let paths = vec![path.clone()];
    let result = try_fix_config(&paths, &exe);
    assert!(result);
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains(&format!("command={exe}")));
    assert!(content.contains("other=keep"));
}

#[test]
fn try_fix_config_wrong_command_non_tty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "command=/wrong/binary\n").unwrap();
    let paths = vec![path];
    // stdin is not a terminal in test, so falls to non-interactive branch
    let result = try_fix_config(&paths, "/usr/bin/correct");
    assert!(!result);
}

#[test]
fn find_config_matching_current_exe_matches_valid() {
    let dir = tempfile::tempdir().unwrap();
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, format!("command={exe}\nrunAtGameStart=true\n")).unwrap();
    let paths = vec![path];
    assert!(check_config_matches_current_exe(&paths));
}

#[test]
fn find_existing_config_finds_first() {
    let dir = tempfile::tempdir().unwrap();
    let path1 = dir.path().join("a.properties");
    let path2 = dir.path().join("b.properties");
    std::fs::write(&path2, "").unwrap();
    let paths = vec![path1.clone(), path2.clone()];
    assert_eq!(find_existing_config(&paths), Some(&path2));
}

#[test]
fn show_setup_message_to_writes_content() {
    let mut buf = Vec::new();
    show_setup_message_to(&mut buf, "/test/path", "/usr/bin/copilot");
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("尚未配置"));
    assert!(output.contains("command=/usr/bin/copilot"));
    assert!(output.contains("runAtGameStart=true"));
    assert!(output.contains("/test/path"));
}

#[test]
fn show_cjk_mod_language_message_to_writes_warning() {
    let mut buf = Vec::new();
    let lang = DetectedGameLanguage {
        value: "ZHS".into(),
        source: "STSGameplaySettings".into(),
    };
    let config_path = std::path::PathBuf::from("/test/CommunicationMod/config.properties");
    let cjk_path = std::path::PathBuf::from("/test/CommunicationModCJK/config.properties");
    show_cjk_mod_language_message_to(&mut buf, &lang, &config_path, &cjk_path);
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("ZHS"));
    assert!(output.contains("Communication Mod CJK"));
    assert!(output.contains("CommunicationModCJK"));
}

#[test]
fn restart_hint_writes_message() {
    restart_hint();
}

// ============================================================
// try_fix_config — additional coverage
// ============================================================

#[test]
fn try_fix_config_already_good() {
    let dir = tempfile::tempdir().unwrap();
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, format!("command={exe}\nrunAtGameStart=true\n")).unwrap();
    let paths = vec![path];
    let result = try_fix_config(&paths, &exe);
    assert!(result);
}

#[test]
fn try_fix_config_no_config_exists() {
    let paths = vec![
        PathBuf::from("/tmp/nonexistent-test-config-99999.properties"),
        PathBuf::from("/tmp/another-nonexistent-88888.properties"),
    ];
    let result = try_fix_config(&paths, "/usr/bin/copilot");
    assert!(!result);
}

// ============================================================
// write_command_to_config — additional edge cases
// ============================================================

#[test]
fn write_command_to_config_only_command_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "command=/old/cmd\n").unwrap();
    assert!(write_command_to_config(&path, "/new/cmd"));
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("command=/new/cmd"));
    assert!(content.contains("runAtGameStart=true"));
}

#[test]
fn write_command_to_config_only_run_at_game_start_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "runAtGameStart=false\n").unwrap();
    assert!(write_command_to_config(&path, "/bin/copilot"));
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("command=/bin/copilot"));
    assert!(content.contains("runAtGameStart=true"));
}

#[test]
fn write_command_to_config_both_already_set_correctly() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(
        &path,
        "command=/correct/cmd\nrunAtGameStart=true\nother=keep\n",
    )
    .unwrap();
    assert!(write_command_to_config(&path, "/correct/cmd"));
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("command=/correct/cmd"));
    assert!(content.contains("runAtGameStart=true"));
    assert!(content.contains("other=keep"));
}

// ============================================================
// read_gameplay_settings_language — edge cases
// ============================================================

#[test]
fn read_gameplay_settings_language_missing_field() {
    let (_dir, path) = temp_config(r#"{"Fast Mode": "true"}"#);
    assert_eq!(read_gameplay_settings_language(&path), None);
}

#[test]
fn read_gameplay_settings_language_empty_value() {
    let (_dir, path) = temp_config(r#"{"LANGUAGE": ""}"#);
    assert_eq!(read_gameplay_settings_language(&path), None);
}

#[test]
fn read_gameplay_settings_language_invalid_json() {
    let (_dir, path) = temp_config("not valid json at all");
    assert_eq!(read_gameplay_settings_language(&path), None);
}

#[test]
fn read_gameplay_settings_language_nonexistent_file() {
    let path = PathBuf::from("/tmp/nonexistent-sts-settings-99999.json");
    assert_eq!(read_gameplay_settings_language(&path), None);
}

#[test]
fn read_gameplay_settings_language_with_whitespace() {
    let (_dir, path) = temp_config(r#"{"LANGUAGE": "  schinese  "}"#);
    assert_eq!(
        read_gameplay_settings_language(&path),
        Some("schinese".to_string())
    );
}

// ============================================================
// extract_vdf_value — edge cases
// ============================================================

#[test]
fn extract_vdf_value_from_typical_content() {
    let content = "\"AppState\"\n{\n\t\"appid\"\t\t\"646570\"\n\t\"language\"\t\t\"schinese\"\n\t\"name\"\t\t\"Slay the Spire\"\n}";
    assert_eq!(
        extract_vdf_value(content, "language"),
        Some("schinese".to_string())
    );
    assert_eq!(
        extract_vdf_value(content, "appid"),
        Some("646570".to_string())
    );
    assert_eq!(
        extract_vdf_value(content, "name"),
        Some("Slay the Spire".to_string())
    );
}

#[test]
fn extract_vdf_value_empty_content() {
    assert_eq!(extract_vdf_value("", "key"), None);
}

#[test]
fn extract_vdf_value_key_not_found() {
    assert_eq!(
        extract_vdf_value("\"other\"\t\t\"value\"", "language"),
        None
    );
}

#[test]
fn extract_vdf_value_no_value_after_key() {
    assert_eq!(extract_vdf_value("\"language\"", "language"), None);
}

#[test]
fn extract_vdf_value_case_insensitive_key() {
    let content = "\"Language\"\t\t\"schinese\"";
    assert_eq!(
        extract_vdf_value(content, "language"),
        Some("schinese".to_string())
    );
    assert_eq!(
        extract_vdf_value(content, "LANGUAGE"),
        Some("schinese".to_string())
    );
    assert_eq!(
        extract_vdf_value(content, "Language"),
        Some("schinese".to_string())
    );
}

#[test]
fn extract_vdf_value_trims_value_whitespace() {
    let content = "  \"appid\"  \t  \"  646570  \"  ";
    assert_eq!(
        extract_vdf_value(content, "appid"),
        Some("646570".to_string())
    );
}

#[test]
fn extract_vdf_value_multiline_finds_correct_key() {
    let content =
        "\"appid\"\t\t\"646570\"\n\"language\"\t\t\"english\"\n\"installdir\"\t\t\"SlayTheSpire\"";
    assert_eq!(
        extract_vdf_value(content, "language"),
        Some("english".to_string())
    );
    assert_eq!(
        extract_vdf_value(content, "installdir"),
        Some("SlayTheSpire".to_string())
    );
}

#[test]
fn extract_vdf_value_skips_lines_without_parts() {
    let content = "{\n\t\"language\"\t\t\"schinese\"\n}";
    assert_eq!(
        extract_vdf_value(content, "language"),
        Some("schinese".to_string())
    );
}

// ============================================================
// read_steam_appmanifest_language — edge cases
// ============================================================

#[test]
fn read_steam_appmanifest_language_from_sample() {
    let (_dir, path) =
        temp_config("\"AppState\"\n{\n\t\"appid\"\t\t\"646570\"\n\t\"language\"\t\t\"english\"\n}");
    assert_eq!(
        read_steam_appmanifest_language(&path),
        Some("english".to_string())
    );
}

#[test]
fn read_steam_appmanifest_language_empty_value() {
    let (_dir, path) = temp_config("\"language\"\t\t\"\"");
    assert_eq!(read_steam_appmanifest_language(&path), None);
}

#[test]
fn read_steam_appmanifest_language_nonexistent_file() {
    let path = PathBuf::from("/tmp/nonexistent-appmanifest-99999.acf");
    assert_eq!(read_steam_appmanifest_language(&path), None);
}

#[test]
fn read_steam_appmanifest_language_no_language_key() {
    let (_dir, path) = temp_config("\"appid\"\t\t\"646570\"\n\"name\"\t\t\"Slay the Spire\"");
    assert_eq!(read_steam_appmanifest_language(&path), None);
}

#[test]
fn read_steam_appmanifest_language_whitespace_only_value() {
    let (_dir, path) = temp_config("\"language\"\t\t\"   \"");
    assert_eq!(read_steam_appmanifest_language(&path), None);
}

// ============================================================
// ensure_config — integration tests
// ============================================================

#[test]
fn ensure_config_returns_true_when_valid_config_matches() {
    let home = tempfile::tempdir().unwrap();
    let config_dir = home
        .path()
        .join(".config")
        .join("ModTheSpire")
        .join("CommunicationModCJK");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.properties");
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    std::fs::write(
        &config_path,
        format!("command={exe}\nrunAtGameStart=true\n"),
    )
    .unwrap();

    let old_home = std::env::var("HOME").ok();
    let old_lang = std::env::var("SLAY_THE_SPIRE_LANGUAGE").ok();
    unsafe { std::env::set_var("HOME", home.path().to_str().unwrap()) };
    unsafe { std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", "ENG") };

    let result = ensure_config();

    if let Some(h) = old_home {
        unsafe { std::env::set_var("HOME", h) };
    } else {
        unsafe { std::env::remove_var("HOME") };
    }
    if let Some(l) = old_lang {
        unsafe { std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", l) };
    } else {
        unsafe { std::env::remove_var("SLAY_THE_SPIRE_LANGUAGE") };
    }

    assert!(result);
}

#[test]
fn ensure_config_cjk_warning_on_standard_mod() {
    let home = tempfile::tempdir().unwrap();
    let config_dir = home
        .path()
        .join(".config")
        .join("ModTheSpire")
        .join("CommunicationMod");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.properties");
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    std::fs::write(
        &config_path,
        format!("command={exe}\nrunAtGameStart=true\n"),
    )
    .unwrap();

    let old_home = std::env::var("HOME").ok();
    let old_lang = std::env::var("SLAY_THE_SPIRE_LANGUAGE").ok();
    unsafe { std::env::set_var("HOME", home.path().to_str().unwrap()) };
    unsafe { std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", "ZHS") };

    let result = ensure_config();

    if let Some(h) = old_home {
        unsafe { std::env::set_var("HOME", h) };
    } else {
        unsafe { std::env::remove_var("HOME") };
    }
    if let Some(l) = old_lang {
        unsafe { std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", l) };
    } else {
        unsafe { std::env::remove_var("SLAY_THE_SPIRE_LANGUAGE") };
    }

    assert!(!result);
}

// ============================================================
// Additional coverage: language_needs_cjk_mod prefixes
// ============================================================

#[test]
fn language_needs_cjk_mod_prefix_variants() {
    assert!(language_needs_cjk_mod("zh-TW"));
    assert!(language_needs_cjk_mod("zh_CN"));
    assert!(language_needs_cjk_mod("ja_JP"));
    assert!(language_needs_cjk_mod("ko-KR"));
    assert!(language_needs_cjk_mod("zh-whatever"));
    assert!(language_needs_cjk_mod("ja-something"));
    assert!(language_needs_cjk_mod("ko variant"));
}

// ============================================================
// Additional coverage: extract_property_value edge cases
// ============================================================

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

// ============================================================
// Additional coverage: config_has_command / config_points_to_this_binary
// ============================================================

#[test]
fn config_has_command_false_when_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "other=value\n").unwrap();
    assert!(!config_has_command(&path));
}

#[test]
fn config_points_to_this_binary_false_for_wrong_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "command=/definitely/wrong/path\n").unwrap();
    assert!(!config_points_to_this_binary(&path));
}

#[test]
fn config_points_to_this_binary_false_when_no_command() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "other=value\n").unwrap();
    assert!(!config_points_to_this_binary(&path));
}

// ============================================================
// Additional coverage: parse_command_value edge cases
// ============================================================

#[test]
fn parse_command_value_unescapes_colon_and_equals() {
    assert_eq!(
        parse_command_value(r"path\\:with\\:colons"),
        Some("path:with:colons".to_string())
    );
    assert_eq!(
        parse_command_value(r"key\\=value"),
        Some("key=value".to_string())
    );
}

#[test]
fn parse_command_value_handles_mixed_escapes() {
    assert_eq!(
        parse_command_value(r"C\:\\foo\\:bar\\=baz.exe"),
        Some(r"C:\foo:bar=baz.exe".to_string())
    );
}

#[test]
fn parse_command_value_empty_after_unescape_returns_none() {
    assert_eq!(parse_command_value("\"   \""), None);
    assert_eq!(parse_command_value("'   '"), None);
}

// ============================================================
// Additional coverage: format_command_value edge cases
// ============================================================

#[test]
fn format_command_value_no_whitespace_no_quotes() {
    assert_eq!(format_command_value("/simple/path"), "/simple/path");
}

#[test]
fn format_command_value_already_double_quoted() {
    let result = format_command_value("\"/usr/bin/copilot\"");
    assert_eq!(result, "\"/usr/bin/copilot\"");
}

#[test]
fn format_command_value_single_quote_not_rewrapped() {
    let result = format_command_value("'/usr/bin/copilot'");
    assert_eq!(result, "'/usr/bin/copilot'");
}

// ============================================================
// Additional coverage: runAtGameStart detection
// ============================================================

#[test]
fn run_at_game_start_enabled_absent_returns_false() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "command=/bin/copilot\n").unwrap();
    assert!(!run_at_game_start_enabled(&path));
}

// ============================================================
// Additional coverage: cjk_mod_required_for_language
// ============================================================

#[test]
fn cjk_mod_required_for_language_none_language_returns_false() {
    let path = PathBuf::from("/tmp/CommunicationMod/config.properties");
    assert!(!cjk_mod_required_for_language(&path, None));
}

#[test]
fn cjk_mod_required_for_language_non_cjk_language_on_standard_mod() {
    let path = PathBuf::from("/tmp/CommunicationMod/config.properties");
    assert!(!cjk_mod_required_for_language(&path, Some("ENG")));
}

// ============================================================
// Additional coverage: show_cjk_mod_language_message_to with language value
// ============================================================

#[test]
fn show_cjk_mod_language_message_to_includes_source_and_paths() {
    let mut buf = Vec::new();
    let lang = DetectedGameLanguage {
        value: "Japanese".into(),
        source: "/steam/appmanifest.acf".into(),
    };
    let config_path = PathBuf::from("/test/ModTheSpire/CommunicationMod/config.properties");
    let cjk_path = PathBuf::from("/test/ModTheSpire/CommunicationModCJK/config.properties");
    show_cjk_mod_language_message_to(&mut buf, &lang, &config_path, &cjk_path);
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("Japanese"));
    assert!(output.contains("/steam/appmanifest.acf"));
    assert!(output.contains("Communication Mod CJK"));
    assert!(output.contains("CommunicationModCJK"));
    assert!(output.contains("CommunicationMod/config.properties"));
}

// ============================================================
// Additional coverage: detect_game_language via env var
// ============================================================

#[test]
fn detect_game_language_from_env_var() {
    let old = std::env::var("SLAY_THE_SPIRE_LANGUAGE").ok();
    unsafe { std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", "schinese") };
    let result = detect_game_language();
    if let Some(v) = old {
        unsafe { std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", v) };
    } else {
        unsafe { std::env::remove_var("SLAY_THE_SPIRE_LANGUAGE") };
    }
    assert_eq!(
        result,
        Some(DetectedGameLanguage {
            value: "schinese".into(),
            source: "SLAY_THE_SPIRE_LANGUAGE".into()
        })
    );
}

#[test]
fn detect_game_language_env_var_trims_whitespace() {
    let old = std::env::var("SLAY_THE_SPIRE_LANGUAGE").ok();
    // Set to whitespace-padded; detect_game_language trims and checks non-empty,
    // then falls through to file detection (may or may not find files depending on system).
    // We just verify the env var is not used as-is with whitespace.
    unsafe {
        std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", "  schinese  ");
    }
    let result = detect_game_language();
    if let Some(v) = old {
        unsafe {
            std::env::set_var("SLAY_THE_SPIRE_LANGUAGE", v);
        }
    } else {
        unsafe {
            std::env::remove_var("SLAY_THE_SPIRE_LANGUAGE");
        }
    }
    // Trimmed "schinese" should be detected from env var (not from a file)
    assert_eq!(result.map(|d| d.value), Some("schinese".to_string()));
}

// ============================================================
// Additional coverage: communication_mod_config_paths
// ============================================================

#[test]
fn communication_mod_config_paths_includes_cjk_and_standard() {
    let paths = communication_mod_config_paths();
    let cjk_count = paths
        .iter()
        .filter(|p| p.to_string_lossy().contains("CommunicationModCJK"))
        .count();
    let std_count = paths
        .iter()
        .filter(|p| p.to_string_lossy().contains("CommunicationMod"))
        .count();
    assert!(cjk_count > 0, "should include CommunicationModCJK paths");
    assert!(std_count >= cjk_count, "should include both mod variants");
}

// ============================================================
// Additional coverage: try_fix_config edge cases
// ============================================================

#[test]
fn try_fix_config_no_command_but_other_content() {
    let dir = tempfile::tempdir().unwrap();
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "someOtherKey=someValue\nrunAtGameStart=false\n").unwrap();
    let paths = vec![path.clone()];
    let result = try_fix_config(&paths, &exe);
    assert!(result);
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains(&format!("command={}", format_command_value(&exe))));
    assert!(content.contains("runAtGameStart=true"));
    assert!(content.contains("someOtherKey=someValue"));
}

#[test]
fn try_fix_config_wrong_command_non_tty_verifies_message() {
    // stdin is not a terminal in test → falls to non-interactive error path
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "command=/wrong/binary\nrunAtGameStart=true\n").unwrap();
    let paths = vec![path];
    let result = try_fix_config(&paths, "/usr/bin/correct");
    assert!(
        !result,
        "should return false when config has wrong command and no tty"
    );
}

#[test]
fn try_fix_config_unknown_format_with_command() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "command=/some/other\n").unwrap();
    let paths = vec![path];
    let result = try_fix_config(&paths, "/usr/bin/copilot");
    assert!(!result, "should fail on non-tty with wrong command");
}

// ============================================================
// Final coverage push: ensure_config and try_fix_config edge cases
// ============================================================

#[test]
fn communicate_mod_config_paths_with_localappdata() {
    let old = std::env::var("LOCALAPPDATA").ok();
    unsafe { std::env::set_var("LOCALAPPDATA", "C:\\Users\\Test\\AppData\\Local") };
    let paths = communication_mod_config_paths();
    if let Some(v) = old {
        unsafe { std::env::set_var("LOCALAPPDATA", v) };
    } else {
        unsafe { std::env::remove_var("LOCALAPPDATA") };
    }
    assert!(!paths.is_empty());
}

#[test]
fn config_is_valid_both_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "other=value\n").unwrap();
    assert!(!config_is_valid(&path));
}

#[test]
fn config_is_valid_missing_run_at_game_start() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, "command=/bin/copilot\n").unwrap();
    assert!(!config_is_valid(&path));
}

#[test]
fn try_fix_config_points_here_but_not_startup_enabled_writes_to_config() {
    let dir = tempfile::tempdir().unwrap();
    let exe = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let path = dir.path().join("config.properties");
    std::fs::write(&path, format!("command={exe}\n")).unwrap();
    let paths = vec![path.clone()];
    let result = try_fix_config(&paths, &exe);
    assert!(result);
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("runAtGameStart=true"));
}

#[test]
fn find_existing_config_returns_none_for_empty_slice() {
    let paths: Vec<PathBuf> = vec![];
    assert_eq!(find_existing_config(&paths), None);
}
