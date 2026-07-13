use super::*;

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
fn empty_command_auto_fixes() {
    let current = env::current_exe().unwrap().display().to_string();
    let (_dir, config) = temp_config("command=\n");
    assert!(write_command_to_config(&config, &current));
    let updated = fs::read_to_string(&config).unwrap();
    assert!(updated.contains(&format!("command={}", format_command_value(&current))));
    assert!(updated.contains("runAtGameStart=true"));
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
    assert!(content.contains(&format!("command={}", format_command_value(&exe))));
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
