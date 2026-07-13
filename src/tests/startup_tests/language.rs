use super::*;

#[test]
fn communication_mod_cjk_config_dir_is_preferred() {
    assert_eq!(COMMUNICATION_MOD_CONFIG_DIRS[0], "CommunicationModCJK");
    assert!(COMMUNICATION_MOD_CONFIG_DIRS.contains(&"CommunicationMod"));
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
fn language_needs_cjk_mod_prefix_variants() {
    assert!(language_needs_cjk_mod("zh-TW"));
    assert!(language_needs_cjk_mod("zh_CN"));
    assert!(language_needs_cjk_mod("ja_JP"));
    assert!(language_needs_cjk_mod("ko-KR"));
    assert!(language_needs_cjk_mod("zh-whatever"));
    assert!(language_needs_cjk_mod("ja-something"));
    assert!(language_needs_cjk_mod("ko variant"));
}

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
