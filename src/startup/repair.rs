use super::{config, messages};
use std::io::{BufRead, IsTerminal, Write};
use std::path::PathBuf;

/// Attempts to fix the CommunicationMod config. Returns true if the config was updated
/// and is now correct (but caller should still exit so CommunicationMod can restart).
pub(super) fn try_fix_config(paths: &[PathBuf], exe_path: &str) -> bool {
    let tty = std::io::stdin().is_terminal();

    if let Some(existing) = config::find_existing_config(paths) {
        let has_command = config::config_has_command(existing);
        let points_here = config::config_points_to_this_binary(existing);
        let startup_enabled = config::run_at_game_start_enabled(existing);

        if points_here && startup_enabled {
            return true;
        }

        if points_here && !startup_enabled {
            if config::write_command_to_config(existing, exe_path) {
                let mut stdout = std::io::stdout().lock();
                let _ = writeln!(
                    stdout,
                    "检测到 Communication Mod CJK 未启用启动命令，已自动设置 runAtGameStart=true"
                );
                let _ = stdout.flush();
                messages::restart_hint();
                tracing::info!("auto-enabled runAtGameStart in config");
                return true;
            }
            tracing::error!("failed to write config file");
            return false;
        }

        if !has_command {
            if config::write_command_to_config(existing, exe_path) {
                let mut stdout = std::io::stdout().lock();
                let _ = writeln!(stdout, "检测到空配置，已自动更新 command={exe_path}");
                let _ = stdout.flush();
                messages::restart_hint();
                tracing::info!("auto-updated empty command in config");
                return true;
            }
            tracing::error!("failed to write config file");
            return false;
        }

        if let Some(old_command) = config::extract_command_value(existing) {
            if tty {
                let mut stdout = std::io::stdout().lock();
                let _ = write!(
                    stdout,
                    "command 当前指向 {old_command}，是否替换为 {exe_path}？[y/N] "
                );
                let _ = stdout.flush();

                let mut input = String::new();
                let stdin = std::io::stdin();
                if stdin.lock().read_line(&mut input).is_ok() {
                    let answer = input.trim().to_lowercase();
                    if (answer == "y" || answer == "yes")
                        && config::write_command_to_config(existing, exe_path)
                    {
                        let _ = writeln!(stdout, "已更新配置。");
                        let _ = stdout.flush();
                        messages::restart_hint();
                        tracing::info!("user confirmed config update");
                        return true;
                    }
                }
                let _ = writeln!(stdout, "配置未更改。");
                let _ = stdout.flush();
                return false;
            }

            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(
                stdout,
                "配置文件的 command 指向不同路径：\n  当前值：{old_command}\n  期望值：{exe_path}\n\n请手动更新配置文件或传入正确输入重新运行。"
            );
            let _ = stdout.flush();
            return false;
        }
    }

    let config_path = paths
        .first()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "{SpireConfig path}".to_string());
    messages::show_setup_message(&config_path, exe_path);
    tracing::warn!("CommunicationMod config not found at {config_path}");
    false
}
