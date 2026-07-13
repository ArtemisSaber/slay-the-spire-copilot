use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) const COMMUNICATION_MOD_CONFIG_DIRS: &[&str] =
    &["CommunicationModCJK", "CommunicationMod"];

pub(super) fn communication_mod_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(home) = env::var("HOME") {
        for mod_config_dir in COMMUNICATION_MOD_CONFIG_DIRS {
            paths.push(
                PathBuf::from(&home)
                    .join(".config")
                    .join("ModTheSpire")
                    .join(mod_config_dir)
                    .join("config.properties"),
            );
            paths.push(
                PathBuf::from(&home)
                    .join("Library")
                    .join("Preferences")
                    .join("ModTheSpire")
                    .join(mod_config_dir)
                    .join("config.properties"),
            );
        }
    }

    if let Ok(localappdata) = env::var("LOCALAPPDATA") {
        for mod_config_dir in COMMUNICATION_MOD_CONFIG_DIRS {
            paths.push(
                PathBuf::from(&localappdata)
                    .join("ModTheSpire")
                    .join(mod_config_dir)
                    .join("config.properties"),
            );
        }
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    paths
}

pub(super) fn parse_command_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let raw = if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        &value[1..value.len() - 1]
    } else {
        value
    };

    let unescaped = raw
        .replace("\\\\", "\\")
        .replace("\\:", ":")
        .replace("\\=", "=")
        .trim()
        .to_string();

    if unescaped.is_empty() {
        None
    } else {
        Some(unescaped)
    }
}

pub(super) fn extract_command_value(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path)
        .map_err(|error| tracing::warn!("failed to read {}: {error}", path.display()))
        .ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("command=") {
            return parse_command_value(value);
        }
    }
    None
}

pub(super) fn extract_property_value(path: &Path, key: &str) -> Option<String> {
    let content = fs::read_to_string(path)
        .map_err(|error| tracing::warn!("failed to read {}: {error}", path.display()))
        .ok()?;
    let prefix = format!("{key}=");
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix(&prefix) {
            return Some(value.trim().to_string());
        }
    }
    None
}

pub(super) fn paths_equal(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(canonical_a), Ok(canonical_b)) => canonical_a == canonical_b,
        _ => a == b,
    }
}

pub(super) fn config_has_command(path: &Path) -> bool {
    extract_command_value(path).is_some()
}

pub(super) fn run_at_game_start_enabled(path: &Path) -> bool {
    extract_property_value(path, "runAtGameStart").is_some_and(|value| {
        matches!(
            value.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

pub(super) fn config_is_valid(path: &Path) -> bool {
    config_has_command(path) && run_at_game_start_enabled(path)
}

pub(super) fn config_points_to_this_binary(path: &Path) -> bool {
    let Some(command) = extract_command_value(path) else {
        return false;
    };
    let Ok(current) = env::current_exe() else {
        return false;
    };
    paths_equal(&PathBuf::from(&command), &current)
}

pub(super) fn current_exe_string() -> String {
    env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "/path/to/slay-the-spire-copilot".to_string())
}

pub(super) fn config_path_uses_cjk_mod(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case("CommunicationModCJK")
    })
}

pub(super) fn find_config_matching_current_exe(paths: &[PathBuf]) -> Option<&PathBuf> {
    paths
        .iter()
        .find(|path| path.exists() && config_is_valid(path) && config_points_to_this_binary(path))
}

#[cfg(test)]
pub(super) fn check_config_matches_current_exe(paths: &[PathBuf]) -> bool {
    find_config_matching_current_exe(paths).is_some()
}

pub(super) fn find_existing_config(paths: &[PathBuf]) -> Option<&PathBuf> {
    paths.iter().find(|path| path.exists())
}

pub(super) fn format_command_value(command: &str) -> String {
    let escaped = command.replace('\\', "\\\\");
    if escaped.chars().any(char::is_whitespace)
        && !(escaped.starts_with('"') && escaped.ends_with('"'))
    {
        format!("\"{escaped}\"")
    } else {
        escaped
    }
}

pub(super) fn write_command_to_config(path: &Path, command: &str) -> bool {
    let content = fs::read_to_string(path).unwrap_or_default();
    let command = format_command_value(command);

    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    let mut found_command = false;
    let mut found_run_at_game_start = false;

    for line in &mut lines {
        if line.starts_with("command=") {
            *line = format!("command={command}");
            found_command = true;
        }
        if line.starts_with("runAtGameStart=") {
            *line = "runAtGameStart=true".to_string();
            found_run_at_game_start = true;
        }
    }

    if !found_command {
        lines.push(format!("command={command}"));
    }
    if !found_run_at_game_start {
        lines.push("runAtGameStart=true".to_string());
    }

    let mut new_content = lines.join("\n");
    new_content.push('\n');

    fs::write(path, &new_content).is_ok()
}
