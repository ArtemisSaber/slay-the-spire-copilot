use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

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

    paths
}

fn config_is_valid(path: &PathBuf) -> bool {
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            if line.starts_with("command=") {
                let value = line.strip_prefix("command=").unwrap_or("");
                return !value.is_empty();
            }
        }
    }
    false
}

/// Returns true if the CommunicationMod config exists and is properly configured.
pub fn ensure_config() -> bool {
    let paths = communication_mod_config_paths();

    let found = paths.iter().any(|p| p.exists() && config_is_valid(p));

    if found {
        tracing::info!("CommunicationMod config found and configured");
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
         尚未找到 CommunicationMod 配置。请按以下步骤操作：\n\n\
         1. 安装 ModTheSpire 和 CommunicationMod\n\
            https://github.com/kiooeht/ModTheSpire\n\
            https://github.com/ForgottenArbiter/CommunicationMod\n\n\
         2. 编辑配置文件：\n\
            {config_path}\n\n\
         3. 添加以下内容：\n\
            command={exe_path}\n\
            runAtGameStart=true\n\n\
         4. 通过 ModTheSpire 启动游戏并启用 CommunicationMod\n\n\
         也可以手动测试（mock provider）：\n\
           echo '{{\"in_game\":true,...}}' | {exe_path}\n"
    );

    // Write to stdout for the user to see
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "{message}");
    let _ = stdout.flush();

    tracing::warn!("CommunicationMod config not found at {config_path}");
    tracing::info!("To configure, add: command={exe_path}");

    false
}
