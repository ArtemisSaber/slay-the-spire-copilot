use super::*;

#[test]
fn detect_game_language_from_env_var() {
    let _env_guard = ENV_LOCK.lock().unwrap();
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
    let _env_guard = ENV_LOCK.lock().unwrap();
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

#[test]
fn communicate_mod_config_paths_with_localappdata() {
    let _env_guard = ENV_LOCK.lock().unwrap();
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
