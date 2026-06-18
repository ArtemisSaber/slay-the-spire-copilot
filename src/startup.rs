use std::env;
use std::fs;
use std::path::PathBuf;

fn communication_mod_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Linux
    if let Ok(home) = env::var("HOME") {
        paths.push(
            PathBuf::from(home)
                .join(".config")
                .join("ModTheSpire")
                .join("CommunicationMod")
                .join("config.properties"),
        );
    }

    // Windows
    if let Ok(localappdata) = env::var("LOCALAPPDATA") {
        paths.push(
            PathBuf::from(localappdata)
                .join("ModTheSpire")
                .join("CommunicationMod")
                .join("config.properties"),
        );
    }

    // Mac
    if let Ok(home) = env::var("HOME") {
        paths.push(
            PathBuf::from(home)
                .join("Library")
                .join("Preferences")
                .join("ModTheSpire")
                .join("CommunicationMod")
                .join("config.properties"),
        );
    }

    paths
}

pub fn ensure_config() {
    let paths = communication_mod_config_paths();

    let found = paths.iter().any(|p| {
        if !p.exists() {
            return false;
        }
        if let Ok(content) = fs::read_to_string(p) {
            // Check if the command= line has a value
            for line in content.lines() {
                if line.starts_with("command=") {
                    let value = line.strip_prefix("command=").unwrap_or("");
                    if !value.is_empty() {
                        return true;
                    }
                }
            }
        }
        false
    });

    if found {
        tracing::info!("CommunicationMod config found and configured");
    } else {
        tracing::warn!("CommunicationMod config not found or command= is empty");
        tracing::info!(
            "Expected config at: {}",
            paths
                .first()
                .map(|p| p.display().to_string())
                .unwrap_or_else(
                    || "~/.config/ModTheSpire/CommunicationMod/config.properties".to_string()
                )
        );

        // Write instructions to output/advice.txt so the user sees it
        let output_dir = std::path::Path::new("output");
        let _ = fs::create_dir_all(output_dir);
        let path = output_dir.join("advice.txt");

        let instructions = format!(
            "============================================================\n\
             Slay the Spire AI Copilot\n\
             尚未配置 CommunicationMod\n\
             ============================================================\n\n\
             配置步骤：\n\
             1. 安装 ModTheSpire 和 CommunicationMod\n\
                https://github.com/kiooeht/ModTheSpire\n\
                https://github.com/ForgottenArbiter/CommunicationMod\n\n\
             2. 编辑配置文件：\n\
                {}\n\n\
             3. 添加以下内容（替换为你的二进制路径）：\n\
                command={}\n\
                runAtGameStart=true\n\n\
             4. 通过 ModTheSpire 启动游戏并启用 CommunicationMod\n\n\
             当前使用 mock provider 进行本地测试。\n",
            paths
                .first()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "{SpireConfig路径}".to_string()),
            env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "/path/to/slay-the-spire-copilot".to_string()),
        );

        let _ = fs::write(&path, &instructions);
    }
}
