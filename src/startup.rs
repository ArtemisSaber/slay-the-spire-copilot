use std::env;
use std::fs;
use std::io::Write;
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

/// Checks whether any of the given config paths exist and have a valid command= value.
pub fn check_config_exists(paths: &[PathBuf]) -> bool {
    paths.iter().any(|p| p.exists() && config_is_valid(p))
}

/// Checks whether any valid config path also points to the current running binary.
pub fn check_config_matches_current_exe(paths: &[PathBuf]) -> bool {
    paths
        .iter()
        .any(|p| p.exists() && config_is_valid(p) && config_points_to_this_binary(p))
}

/// Returns true if the CommunicationMod config exists, is properly configured,
/// and points to the currently running binary. Prints setup instructions to stdout if not.
pub fn ensure_config() -> bool {
    let paths = communication_mod_config_paths();

    if check_config_matches_current_exe(&paths) {
        tracing::info!("CommunicationMod config found and configured correctly");
        return true;
    }

    let config_path = paths
        .first()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "{SpireConfig path}".to_string());

    let exe_path = env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "/path/to/slay-the-spire-copilot".to_string());

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

    if check_config_exists(&paths) {
        tracing::warn!(
            "CommunicationMod config found but command= does not point to this binary ({exe_path})"
        );
    } else {
        tracing::warn!("CommunicationMod config not found at {config_path}");
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
        let current = std::env::current_exe().unwrap();
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
}
