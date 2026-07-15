use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;

use super::types::{EnvAssignment, LearningSetup, PLACEHOLDER_API_KEY, parse_env_bool};

#[cfg(test)]
pub(crate) fn env_file_needs_setup(path: &Path) -> bool {
    if !path.exists() {
        return true;
    }

    let values = read_env_values(path);
    api_values_need_setup(&values) || feature_values_need_setup(&values)
}

pub(crate) fn api_values_need_setup(values: &HashMap<String, String>) -> bool {
    match provider_value(values) {
        Some("mock") => false,
        Some("pollinations-free") => missing_value(values, "LLM_BASE_URL"),
        Some("openai-compatible" | "anthropic") => {
            missing_value(values, "LLM_BASE_URL")
                || values
                    .get("LLM_API_KEY")
                    .is_none_or(|value| missing_or_placeholder(value))
        }
        Some(_) => true,
        None => true,
    }
}

pub(crate) fn feature_values_need_setup(values: &HashMap<String, String>) -> bool {
    let Some(auto_play) = parse_env_bool(values.get("AUTO_PLAY").map(String::as_str)) else {
        return true;
    };
    let Some(learning) = LearningSetup::from_env(values.get("MEMORY_MODE").map(String::as_str))
    else {
        return true;
    };
    !auto_play && learning != LearningSetup::Off
}

pub(crate) fn provider_value(values: &HashMap<String, String>) -> Option<&str> {
    values.get("LLM_PROVIDER").map(String::as_str)
}

pub(crate) fn read_env_values(path: &Path) -> HashMap<String, String> {
    fs::read_to_string(path)
        .map(|content| parse_env_values(&content))
        .unwrap_or_default()
}

pub(crate) fn parse_env_values(content: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        values.insert(key.to_string(), parse_env_value(value.trim()));
    }
    values
}

pub(crate) fn parse_env_value(value: &str) -> String {
    if let Some(quoted) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        quoted.replace("\\\"", "\"").replace("\\\\", "\\")
    } else if let Some(quoted) = value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')) {
        quoted.to_string()
    } else {
        value.to_string()
    }
}

pub(crate) fn write_env_assignments(path: &Path, assignments: &[EnvAssignment]) -> io::Result<()> {
    let assignments_by_key: HashMap<&str, &str> = assignments
        .iter()
        .map(|assignment| (assignment.key, assignment.value.as_str()))
        .collect();
    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut seen = HashSet::new();
    let mut lines = Vec::new();

    for line in existing.lines() {
        if let Some(key) = env_line_key(line)
            && let Some(value) = assignments_by_key.get(key)
        {
            lines.push(format!("{key}={}", format_env_value(value)));
            seen.insert(key.to_string());
            continue;
        }
        lines.push(line.to_string());
    }

    if !lines.is_empty() && !lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.push(String::new());
    }

    for assignment in assignments {
        if seen.insert(assignment.key.to_string()) {
            lines.push(format!(
                "{}={}",
                assignment.key,
                format_env_value(&assignment.value)
            ));
        }
    }

    let mut content = lines.join("\n");
    content.push('\n');

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

pub(crate) fn env_line_key(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return None;
    }
    let (key, _) = trimmed.split_once('=')?;
    let key = key.trim();
    (!key.is_empty()).then_some(key)
}

pub(crate) fn format_env_value(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':' | '+'))
    {
        return value.to_string();
    }
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

pub(crate) fn missing_value(values: &HashMap<String, String>, key: &str) -> bool {
    values.get(key).is_none_or(|value| value.trim().is_empty())
}

pub(crate) fn missing_or_placeholder(value: &str) -> bool {
    value.trim().is_empty() || is_placeholder_api_key(value)
}

pub(crate) fn is_placeholder_api_key(value: &str) -> bool {
    value.trim() == PLACEHOLDER_API_KEY
}
