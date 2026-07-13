use super::*;

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
