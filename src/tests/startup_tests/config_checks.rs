use super::*;

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
fn no_config_at_all() {
    let paths = vec![PathBuf::from("/tmp/nonexistent-config-12345.properties")];
    assert!(!check_config_matches_current_exe(&paths));
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
fn find_existing_config_returns_none_for_empty_slice() {
    let paths: Vec<PathBuf> = vec![];
    assert_eq!(find_existing_config(&paths), None);
}
