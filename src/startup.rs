use std::env;
use std::fs;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

fn communication_mod_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(home) = env::var("HOME") {
        paths.push(
            PathBuf::from(&home)
                .join(".config")
                .join("ModTheSpire")
                .join("CommunicationMod")
                .join("config.properties"),
        );
        paths.push(
            PathBuf::from(&home)
                .join("Library")
                .join("Preferences")
                .join("ModTheSpire")
                .join("CommunicationMod")
                .join("config.properties"),
        );
    }

    if let Ok(localappdata) = env::var("LOCALAPPDATA") {
        paths.push(
            PathBuf::from(localappdata)
                .join("ModTheSpire")
                .join("CommunicationMod")
                .join("config.properties"),
        );
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    paths
}

fn extract_command_value(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("command=") {
            let value = value.trim();
            if !value.is_empty() {
                let first_token = value.split_whitespace().next().unwrap_or("");
                return Some(first_token.to_string());
            }
        }
    }
    None
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
    }
}

fn config_is_valid(path: &Path) -> bool {
    extract_command_value(path).is_some()
}

fn config_points_to_this_binary(path: &Path) -> bool {
    let Some(cmd) = extract_command_value(path) else {
        return false;
    };
    let Ok(current) = env::current_exe() else {
        return false;
    };
    paths_equal(&PathBuf::from(&cmd), &current)
}

fn current_exe_string() -> String {
    env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "/path/to/slay-the-spire-copilot".to_string())
}

#[allow(dead_code)]
pub fn check_config_exists(paths: &[PathBuf]) -> bool {
    paths.iter().any(|p| p.exists() && config_is_valid(p))
}

pub fn check_config_matches_current_exe(paths: &[PathBuf]) -> bool {
    paths
        .iter()
        .any(|p| p.exists() && config_is_valid(p) && config_points_to_this_binary(p))
}

fn find_existing_config(paths: &[PathBuf]) -> Option<&PathBuf> {
    paths.iter().find(|p| p.exists())
}

fn write_command_to_config(path: &Path, command: &str) -> bool {
    let content = fs::read_to_string(path).unwrap_or_default();

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut found = false;

    for line in &mut lines {
        if line.starts_with("command=") {
            *line = format!("command={command}");
            found = true;
            break;
        }
    }

    if !found {
        lines.push(format!("command={command}"));
    }

    // Ensure newline at end
    let mut new_content = lines.join("\n");
    new_content.push('\n');

    fs::write(path, &new_content).is_ok()
}

fn show_setup_message(config_path: &str, exe_path: &str) {
    let message = format!(
        "\n\
         ╔═══════════════════════════════════════════════╗\n\
         ║   Slay the Spire AI Copilot — 尚未配置       ║\n\
         ╚═══════════════════════════════════════════════╝\n\n\
         尚未找到指向当前二进制文件的 CommunicationMod 配置。\n\n\
         配置步骤：\n\
         1. 安装 ModTheSpire 和 CommunicationMod\n\
            https://github.com/kiooeht/ModTheSpire\n\
            https://github.com/ForgottenArbiter/CommunicationMod\n\n\
         2. 编辑配置文件：\n\
            {config_path}\n\n\
         3. 添加以下内容（确保 command 指向当前二进制文件）：\n\
            command={exe_path}\n\
            runAtGameStart=true\n\n\
         4. 通过 ModTheSpire 启动游戏并启用 CommunicationMod\n\n\
         也可以手动测试（mock provider）：\n\
           echo '{{\"in_game\":true,...}}' | {exe_path}\n"
    );

    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "{message}");
    let _ = stdout.flush();
}

fn restart_hint() {
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "请通过 ModTheSpire 重新启动游戏以加载新配置。");
    let _ = stdout.flush();
}

/// Attempts to fix the CommunicationMod config. Returns true if the config was updated
/// and is now correct (but caller should still exit so CommunicationMod can restart).
fn try_fix_config(paths: &[PathBuf], exe_path: &str) -> bool {
    let tty = std::io::stdin().is_terminal();

    // Case: config file exists but command= is empty or missing
    if let Some(existing) = find_existing_config(paths) {
        let has_valid_command = config_is_valid(existing);
        let points_here = config_points_to_this_binary(existing);

        if points_here {
            return true; // already good, shouldn't reach here
        }

        if !has_valid_command {
            // Empty command → auto-fix
            if write_command_to_config(existing, exe_path) {
                let mut stdout = std::io::stdout().lock();
                let _ = writeln!(stdout, "检测到空配置，已自动更新 command={exe_path}");
                let _ = stdout.flush();
                restart_hint();
                tracing::info!("auto-updated empty command in config");
                return true;
            }
            tracing::error!("failed to write config file");
            return false;
        }

        // Has command but points to wrong binary
        if let Some(old_cmd) = extract_command_value(existing) {
            if tty {
                let mut stdout = std::io::stdout().lock();
                let _ = write!(
                    stdout,
                    "command 当前指向 {old_cmd}，是否替换为 {exe_path}？[y/N] "
                );
                let _ = stdout.flush();

                let mut input = String::new();
                let stdin = std::io::stdin();
                if stdin.lock().read_line(&mut input).is_ok() {
                    let answer = input.trim().to_lowercase();
                    if (answer == "y" || answer == "yes")
                        && write_command_to_config(existing, exe_path)
                    {
                        let _ = writeln!(stdout, "已更新配置。");
                        let _ = stdout.flush();
                        restart_hint();
                        tracing::info!("user confirmed config update");
                        return true;
                    }
                }
                let _ = writeln!(stdout, "配置未更改。");
                let _ = stdout.flush();
                return false;
            }

            // Non-interactive → show what's wrong
            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(
                stdout,
                "配置文件的 command 指向不同路径：\n  当前值：{old_cmd}\n  期望值：{exe_path}\n\n请手动更新配置文件或传入正确输入重新运行。"
            );
            let _ = stdout.flush();
            return false;
        }
    }

    // Case: no config file found at all
    let config_path = paths
        .first()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "{SpireConfig path}".to_string());
    show_setup_message(&config_path, exe_path);
    tracing::warn!("CommunicationMod config not found at {config_path}");
    false
}

/// Returns true if the CommunicationMod config is properly configured and points to
/// this binary. If not, attempts to fix the config (auto-fix or interactive prompt).
pub fn ensure_config() -> bool {
    let paths = communication_mod_config_paths();

    if check_config_matches_current_exe(&paths) {
        tracing::info!("CommunicationMod config found and configured correctly");
        return true;
    }

    let exe = current_exe_string();
    if try_fix_config(&paths, &exe) {
        return false; // config was fixed but need CommunicationMod restart
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_command_from_valid_config() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        fs::write(&config, "command=/usr/bin/my-bot\nrunAtGameStart=true\n").unwrap();
        assert_eq!(
            extract_command_value(&config),
            Some("/usr/bin/my-bot".to_string())
        );
    }

    #[test]
    fn extract_command_strips_args() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        fs::write(&config, "command=/usr/bin/bot --verbose --port 8080\n").unwrap();
        assert_eq!(
            extract_command_value(&config),
            Some("/usr/bin/bot".to_string())
        );
    }

    #[test]
    fn extract_command_empty_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        fs::write(&config, "# comment\ncommand=\n").unwrap();
        assert_eq!(extract_command_value(&config), None);
    }

    #[test]
    fn extract_command_no_line_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        fs::write(&config, "# just a comment\n").unwrap();
        assert_eq!(extract_command_value(&config), None);
    }

    #[test]
    fn check_config_empty_list_returns_false() {
        assert!(!check_config_exists(&[]));
    }

    #[test]
    fn check_config_nonexistent_path_returns_false() {
        let paths = vec![PathBuf::from("/nonexistent/path/config.properties")];
        assert!(!check_config_exists(&paths));
    }

    #[test]
    fn config_is_valid_with_command_set() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        fs::write(&config, "command=/usr/bin/my-bot\nrunAtGameStart=true\n").unwrap();
        assert!(config_is_valid(&config));
    }

    #[test]
    fn check_config_finds_valid_config_among_paths() {
        let dir = tempfile::tempdir().unwrap();
        let valid = dir.path().join("valid.properties");
        let invalid = dir.path().join("invalid.properties");
        let missing = dir.path().join("missing.properties");

        fs::write(&valid, "command=./bot\n").unwrap();
        fs::write(&invalid, "command=\n").unwrap();

        let paths = vec![missing, invalid, valid.clone()];
        assert!(check_config_exists(&paths));
    }

    #[test]
    fn check_config_multiple_paths_none_valid() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.properties");
        let b = dir.path().join("b.properties");

        fs::write(&a, "command=\n").unwrap();

        let paths = vec![a, b];
        assert!(!check_config_exists(&paths));
    }

    #[test]
    fn check_config_matches_current_exe_with_correct_path() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        let current = env::current_exe().unwrap();
        fs::write(&config, format!("command={}\n", current.display())).unwrap();

        let paths = vec![config];
        assert!(check_config_matches_current_exe(&paths));
    }

    #[test]
    fn check_config_matches_current_exe_wrong_path() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        fs::write(&config, "command=/usr/bin/other-bot\n").unwrap();

        let paths = vec![config];
        assert!(check_config_exists(&paths));
        assert!(!check_config_matches_current_exe(&paths));
    }

    #[test]
    fn write_command_updates_existing_line() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        fs::write(&config, "# header\ncommand=/old/bot\nrunAtGameStart=true\n").unwrap();

        assert!(write_command_to_config(&config, "/new/bot"));
        let content = fs::read_to_string(&config).unwrap();
        assert!(content.contains("command=/new/bot"));
        assert!(!content.contains("/old/bot"));
        assert!(content.contains("runAtGameStart=true"));
        assert!(content.contains("# header"));
    }

    #[test]
    fn write_command_appends_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        fs::write(&config, "# just a header\nrunAtGameStart=true\n").unwrap();

        assert!(write_command_to_config(&config, "/new/bot"));
        let content = fs::read_to_string(&config).unwrap();
        assert!(content.contains("command=/new/bot"));
        assert!(content.contains("runAtGameStart=true"));
    }

    #[test]
    fn write_command_handles_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        fs::write(&config, "").unwrap();

        assert!(write_command_to_config(&config, "/new/bot"));
        let content = fs::read_to_string(&config).unwrap();
        assert!(content.contains("command=/new/bot"));
    }

    #[test]
    fn find_existing_config_returns_existing() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("exists.properties");
        let missing = dir.path().join("missing.properties");
        fs::write(&existing, "command=\n").unwrap();

        let paths = vec![missing, existing.clone()];
        assert_eq!(find_existing_config(&paths), Some(&existing));
    }

    #[test]
    fn find_existing_config_returns_none_when_none() {
        let paths = vec![PathBuf::from("/nonexistent")];
        assert_eq!(find_existing_config(&paths), None);
    }

    #[test]
    fn ensure_config_message_contains_required_info() {
        let exe_path = env::current_exe().unwrap();
        let exe = exe_path.display().to_string();

        let message = format!(
            "\n\
             ╔═══════════════════════════════════════════════╗\n\
             ║   Slay the Spire AI Copilot — 尚未配置       ║\n\
             ╚═══════════════════════════════════════════════╝\n\n\
             尚未找到指向当前二进制文件的 CommunicationMod 配置。\n\n\
             配置步骤：\n\
             1. 安装 ModTheSpire 和 CommunicationMod\n\
                https://github.com/kiooeht/ModTheSpire\n\
                https://github.com/ForgottenArbiter/CommunicationMod\n\n\
             2. 编辑配置文件：\n\
                /test/config/path\n\n\
             3. 添加以下内容（确保 command 指向当前二进制文件）：\n\
                command={exe}\n\
                runAtGameStart=true\n\n\
             4. 通过 ModTheSpire 启动游戏并启用 CommunicationMod\n\n\
             也可以手动测试（mock provider）：\n\
               echo '{{\"in_game\":true,...}}' | {exe}\n"
        );

        assert!(message.contains("尚未配置"));
        assert!(message.contains("ModTheSpire"));
        assert!(message.contains("CommunicationMod"));
        assert!(message.contains("command="));
        assert!(message.contains("runAtGameStart=true"));
    }

    #[test]
    fn write_command_preserves_comments_and_structure() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.properties");
        let original = "#Thu Jun 18 22:20:52 SGT 2026\ncommand=old_binary\nrunAtGameStart=true\n";
        fs::write(&config, original).unwrap();

        assert!(write_command_to_config(&config, "new_binary"));
        let content = fs::read_to_string(&config).unwrap();
        assert!(content.contains("#Thu Jun 18 22:20:52 SGT 2026"));
        assert!(content.contains("command=new_binary"));
        assert!(content.contains("runAtGameStart=true"));
        assert!(!content.contains("old_binary"));
    }
}
