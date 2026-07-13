use super::DetectedGameLanguage;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SLAY_THE_SPIRE_APP_ID: &str = "646570";

pub(super) fn push_slay_the_spire_language_path_for_steam_root(
    paths: &mut Vec<PathBuf>,
    steam_root: &Path,
) {
    paths.push(
        steam_root
            .join("steamapps")
            .join("common")
            .join("SlayTheSpire")
            .join("preferences")
            .join("STSGameplaySettings"),
    );
}

pub(super) fn push_steam_appmanifest_path_for_steam_root(
    paths: &mut Vec<PathBuf>,
    steam_root: &Path,
) {
    paths.push(
        steam_root
            .join("steamapps")
            .join(format!("appmanifest_{SLAY_THE_SPIRE_APP_ID}.acf")),
    );
}

pub(super) fn slay_the_spire_language_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(cwd) = env::current_dir() {
        paths.push(cwd.join("preferences").join("STSGameplaySettings"));
    }

    if let Ok(directory) = env::var("SLAY_THE_SPIRE_DIR") {
        paths.push(
            PathBuf::from(directory)
                .join("preferences")
                .join("STSGameplaySettings"),
        );
    }

    if let Ok(steam_dir) = env::var("STEAM_DIR") {
        push_slay_the_spire_language_path_for_steam_root(&mut paths, &PathBuf::from(steam_dir));
    }

    if let Ok(home) = env::var("HOME") {
        let home = PathBuf::from(home);
        paths.push(
            home.join(".local")
                .join("share")
                .join("Steam")
                .join("steamapps")
                .join("common")
                .join("SlayTheSpire")
                .join("preferences")
                .join("STSGameplaySettings"),
        );
        paths.push(
            home.join(".steam")
                .join("steam")
                .join("steamapps")
                .join("common")
                .join("SlayTheSpire")
                .join("preferences")
                .join("STSGameplaySettings"),
        );
        paths.push(
            home.join("Library")
                .join("Application Support")
                .join("Steam")
                .join("steamapps")
                .join("common")
                .join("SlayTheSpire")
                .join("preferences")
                .join("STSGameplaySettings"),
        );
        paths.push(home.join(".prefs").join("STSGameplaySettings"));
    }

    if let Ok(userprofile) = env::var("USERPROFILE") {
        paths.push(
            PathBuf::from(&userprofile)
                .join(".prefs")
                .join("STSGameplaySettings"),
        );
    }

    for variable in [
        "ProgramFiles(x86)",
        "PROGRAMFILES(X86)",
        "ProgramFiles",
        "PROGRAMFILES",
    ] {
        if let Ok(program_files) = env::var(variable) {
            push_slay_the_spire_language_path_for_steam_root(
                &mut paths,
                &PathBuf::from(program_files).join("Steam"),
            );
        }
    }

    paths
}

pub(super) fn steam_appmanifest_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(cwd) = env::current_dir() {
        let parts: Vec<_> = cwd.components().collect();
        if parts.len() >= 3 {
            let library = PathBuf::from_iter(&parts[..parts.len() - 2]);
            paths.push(
                library
                    .join("steamapps")
                    .join(format!("appmanifest_{SLAY_THE_SPIRE_APP_ID}.acf")),
            );
        }
    }

    if let Ok(steam_dir) = env::var("STEAM_DIR") {
        push_steam_appmanifest_path_for_steam_root(&mut paths, &PathBuf::from(steam_dir));
    }

    if let Ok(home) = env::var("HOME") {
        let home = PathBuf::from(home);
        paths.push(
            home.join(".local")
                .join("share")
                .join("Steam")
                .join("steamapps")
                .join(format!("appmanifest_{SLAY_THE_SPIRE_APP_ID}.acf")),
        );
        paths.push(
            home.join(".steam")
                .join("steam")
                .join("steamapps")
                .join(format!("appmanifest_{SLAY_THE_SPIRE_APP_ID}.acf")),
        );
        paths.push(
            home.join("Library")
                .join("Application Support")
                .join("Steam")
                .join("steamapps")
                .join(format!("appmanifest_{SLAY_THE_SPIRE_APP_ID}.acf")),
        );
    }

    for variable in [
        "ProgramFiles(x86)",
        "PROGRAMFILES(X86)",
        "ProgramFiles",
        "PROGRAMFILES",
    ] {
        if let Ok(program_files) = env::var(variable) {
            push_steam_appmanifest_path_for_steam_root(
                &mut paths,
                &PathBuf::from(program_files).join("Steam"),
            );
        }
    }

    paths
}

pub(super) fn read_gameplay_settings_language(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path)
        .map_err(|error| tracing::warn!("failed to read {}: {error}", path.display()))
        .ok()?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|error| tracing::warn!("failed to parse gameplay settings JSON: {error}"))
        .ok()?;
    value
        .get("LANGUAGE")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(super) fn extract_vdf_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let mut parts = line.split('"').filter(|part| !part.trim().is_empty());
        let Some(found_key) = parts.next() else {
            continue;
        };
        let Some(value) = parts.next() else {
            continue;
        };
        if found_key.eq_ignore_ascii_case(key) {
            return Some(value.trim().to_string());
        }
    }
    None
}

pub(super) fn read_steam_appmanifest_language(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path)
        .map_err(|error| tracing::warn!("failed to read {}: {error}", path.display()))
        .ok()?;
    extract_vdf_value(&content, "language").filter(|value| !value.is_empty())
}

pub fn detect_game_language() -> Option<DetectedGameLanguage> {
    if let Ok(language) = env::var("SLAY_THE_SPIRE_LANGUAGE") {
        let language = language.trim();
        if !language.is_empty() {
            return Some(DetectedGameLanguage {
                value: language.to_string(),
                source: "SLAY_THE_SPIRE_LANGUAGE".to_string(),
            });
        }
    }

    for path in slay_the_spire_language_paths() {
        if let Some(language) = read_gameplay_settings_language(&path) {
            return Some(DetectedGameLanguage {
                value: language,
                source: path.display().to_string(),
            });
        }
    }

    for path in steam_appmanifest_paths() {
        if let Some(language) = read_steam_appmanifest_language(&path) {
            return Some(DetectedGameLanguage {
                value: language,
                source: path.display().to_string(),
            });
        }
    }

    None
}

pub(super) fn language_needs_cjk_mod(language: &str) -> bool {
    let language = language
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '_', ' '], "");

    matches!(
        language.as_str(),
        "zhs"
            | "zht"
            | "zh"
            | "zhcn"
            | "zhtw"
            | "schinese"
            | "tchinese"
            | "chinesesimplified"
            | "chinesetraditional"
            | "jpn"
            | "ja"
            | "jp"
            | "japanese"
            | "kor"
            | "ko"
            | "kr"
            | "korean"
            | "koreana"
    ) || language.starts_with("zh")
        || language.starts_with("ja")
        || language.starts_with("ko")
}

pub(super) fn cjk_mod_required_for_language(config_path: &Path, language: Option<&str>) -> bool {
    language.is_some_and(language_needs_cjk_mod)
        && !super::config::config_path_uses_cjk_mod(config_path)
}
