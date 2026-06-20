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
    assert!(output.contains("command="));
    assert!(output.contains("runAtGameStart=true"));
    assert!(output.contains("/test/config/path"));
    assert!(output.contains("/usr/bin/slay-the-spire-copilot"));
}
