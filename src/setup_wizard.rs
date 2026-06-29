use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

const ENV_FILE_NAME: &str = ".env";
const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_POLLINATIONS_FREE_BASE_URL: &str = "https://text.pollinations.ai/openai";
const DEFAULT_POLLINATIONS_FREE_MODEL: &str = "openai-fast";
const DEFAULT_MODEL: &str = "gpt-4o-mini";
const DEFAULT_MAX_TOKENS: &str = "50000";
const DEFAULT_MAX_TOKENS_FAST: &str = "300";
const DEFAULT_TEMPERATURE: &str = "0.7";
const PLACEHOLDER_API_KEY: &str = "sk-your-key-here";

const LLM_ENV_KEYS: &[&str] = &[
    "LLM_PROVIDER",
    "LLM_BASE_URL",
    "LLM_API_KEY",
    "LLM_MODEL",
    "LLM_MODEL_FAST",
    "LLM_MODEL_MEDIUM",
    "LLM_MODEL_HEAVY",
    "LLM_MAX_TOKENS",
    "LLM_MAX_TOKENS_FAST",
    "LLM_MAX_TOKENS_MEDIUM",
    "LLM_MAX_TOKENS_HEAVY",
    "LLM_TEMPERATURE",
    "LLM_DISABLE_FAST_THINKING",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnvAssignment {
    key: &'static str,
    value: String,
}

struct ApiPreset {
    name: &'static str,
    description: &'static str,
    provider: &'static str,
    base_url: &'static str,
    model: &'static str,
    requires_api_key: bool,
    disable_fast_thinking: &'static str,
}

struct ProviderSetup {
    provider: &'static str,
    base_url: String,
    api_key: String,
    fast: String,
    medium: String,
    heavy: String,
    disable_fast_thinking_default: Option<&'static str>,
}

const API_PRESETS: &[ApiPreset] = &[
    ApiPreset {
        name: "Pollinations Free",
        description: "No account or API key, public shared endpoint",
        provider: "pollinations-free",
        base_url: DEFAULT_POLLINATIONS_FREE_BASE_URL,
        model: DEFAULT_POLLINATIONS_FREE_MODEL,
        requires_api_key: false,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "OpenAI",
        description: "BYOK, OpenAI API",
        provider: "openai-compatible",
        base_url: DEFAULT_OPENAI_BASE_URL,
        model: DEFAULT_MODEL,
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "Anthropic Claude",
        description: "BYOK, native Claude Messages API",
        provider: "anthropic",
        base_url: DEFAULT_ANTHROPIC_BASE_URL,
        model: "claude-sonnet-4-6",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "Google Gemini",
        description: "BYOK or free-tier key, OpenAI-compatible Gemini endpoint",
        provider: "openai-compatible",
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        model: "gemini-3.5-flash",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "DeepSeek",
        description: "BYOK, OpenAI-compatible DeepSeek endpoint",
        provider: "openai-compatible",
        base_url: "https://api.deepseek.com",
        model: "deepseek-v4-flash",
        requires_api_key: true,
        disable_fast_thinking: "true",
    },
    ApiPreset {
        name: "Groq",
        description: "Free plan available, OpenAI-compatible Groq endpoint",
        provider: "openai-compatible",
        base_url: "https://api.groq.com/openai/v1",
        model: "llama-3.3-70b-versatile",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "OpenRouter Free Router",
        description: "Free routed models, OpenAI-compatible OpenRouter endpoint",
        provider: "openai-compatible",
        base_url: "https://openrouter.ai/api/v1",
        model: "openrouter/free",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "Mistral",
        description: "BYOK, OpenAI-style chat completions endpoint",
        provider: "openai-compatible",
        base_url: "https://api.mistral.ai/v1",
        model: "mistral-small-latest",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "Cerebras",
        description: "Free account limits available, OpenAI-compatible endpoint",
        provider: "openai-compatible",
        base_url: "https://api.cerebras.ai/v1",
        model: "gpt-oss-120b",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
];

pub fn env_path(project_root: &Path) -> PathBuf {
    project_root.join(ENV_FILE_NAME)
}

pub fn maybe_run_api_setup(project_root: &Path) -> io::Result<bool> {
    let path = env_path(project_root);
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout().lock();
    run_api_setup_with_io(&path, &mut stdin, &mut stdout, false)
}

pub fn run_api_setup(project_root: &Path) -> io::Result<bool> {
    let path = env_path(project_root);
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout().lock();
    run_api_setup_with_io(&path, &mut stdin, &mut stdout, true)
}

fn run_api_setup_with_io(
    env_path: &Path,
    input: &mut impl BufRead,
    output: &mut impl Write,
    force: bool,
) -> io::Result<bool> {
    if !force && !env_file_needs_setup(env_path) {
        return Ok(false);
    }

    writeln!(output)?;
    writeln!(output, "Slay the Spire AI Copilot API Setup")?;
    writeln!(output, "Env file: {}", env_path.display())?;
    writeln!(output)?;

    if !force
        && !prompt_yes_no(
            input,
            output,
            "No API connection is configured. Set it up now?",
            true,
        )?
    {
        writeln!(
            output,
            "Skipping API setup. The mock provider can still run smoke tests."
        )?;
        return Ok(false);
    }

    let existing = read_env_values(env_path);
    let choice = prompt_provider(input, output, &existing)?;
    let Some(assignments) = choice else {
        writeln!(output, "Setup skipped.")?;
        return Ok(false);
    };

    write_env_assignments(env_path, &assignments)?;
    writeln!(output)?;
    writeln!(output, "Saved API configuration to {}", env_path.display())?;
    Ok(true)
}

fn prompt_provider(
    input: &mut impl BufRead,
    output: &mut impl Write,
    existing: &HashMap<String, String>,
) -> io::Result<Option<Vec<EnvAssignment>>> {
    let custom_choice = API_PRESETS.len() + 1;
    let mock_choice = API_PRESETS.len() + 2;
    let skip_choice = API_PRESETS.len() + 3;

    loop {
        writeln!(output, "Choose an API connection:")?;
        for (idx, preset) in API_PRESETS.iter().enumerate() {
            writeln!(
                output,
                "  {}) {} - {}",
                idx + 1,
                preset.name,
                preset.description
            )?;
        }
        writeln!(
            output,
            "  {custom_choice}) Custom OpenAI-compatible endpoint"
        )?;
        writeln!(output, "  {mock_choice}) Mock provider (offline/testing)")?;
        writeln!(output, "  {skip_choice}) Skip")?;

        let default_choice = if provider_value(existing) == Some("mock") {
            mock_choice.to_string()
        } else {
            "1".to_string()
        };

        let choice = prompt_line(input, output, "Selection", Some(&default_choice))?;
        if let Ok(choice_num) = choice.parse::<usize>() {
            if (1..=API_PRESETS.len()).contains(&choice_num) {
                return configure_preset(input, output, existing, &API_PRESETS[choice_num - 1]);
            }
            if choice_num == custom_choice {
                return configure_openai_compatible(input, output, existing);
            }
            if choice_num == mock_choice {
                return Ok(Some(mock_assignments(existing)));
            }
            if choice_num == skip_choice {
                return Ok(None);
            }
        }

        match choice.to_ascii_lowercase().as_str() {
            "custom" => return configure_openai_compatible(input, output, existing),
            "mock" => return Ok(Some(mock_assignments(existing))),
            "q" | "quit" | "skip" => return Ok(None),
            _ => {}
        }

        writeln!(
            output,
            "Please enter a listed number, custom, mock, or skip."
        )?;
        writeln!(output)?;
    }
}

fn configure_preset(
    input: &mut impl BufRead,
    output: &mut impl Write,
    existing: &HashMap<String, String>,
    preset: &ApiPreset,
) -> io::Result<Option<Vec<EnvAssignment>>> {
    writeln!(output)?;
    writeln!(output, "{} preset", preset.name)?;
    writeln!(output, "Base URL: {}", preset.base_url)?;
    if !preset.requires_api_key {
        writeln!(output, "API key: not required")?;
    }

    let api_key = if preset.requires_api_key {
        prompt_api_key(input, output, existing)?
    } else {
        String::new()
    };
    let model = prompt_model_with_default(input, output, existing, preset.model)?;
    let assignments = provider_assignments(
        ProviderSetup {
            provider: preset.provider,
            base_url: preset.base_url.to_string(),
            api_key,
            fast: model.clone(),
            medium: model.clone(),
            heavy: model,
            disable_fast_thinking_default: Some(preset.disable_fast_thinking),
        },
        existing,
    );
    Ok(Some(assignments))
}

fn configure_openai_compatible(
    input: &mut impl BufRead,
    output: &mut impl Write,
    existing: &HashMap<String, String>,
) -> io::Result<Option<Vec<EnvAssignment>>> {
    let default_base_url = existing
        .get("LLM_BASE_URL")
        .map(String::as_str)
        .unwrap_or(DEFAULT_OPENAI_BASE_URL);
    let base_url = prompt_line(input, output, "Base URL", Some(default_base_url))?;
    let api_key = prompt_api_key(input, output, existing)?;
    let model = prompt_model(input, output, existing)?;

    let (fast, medium, heavy) = if prompt_yes_no(
        input,
        output,
        "Configure separate fast/medium/heavy models?",
        false,
    )? {
        let fast = prompt_line(input, output, "Fast model", Some(&model))?;
        let medium = prompt_line(input, output, "Medium model", Some(&model))?;
        let heavy = prompt_line(input, output, "Heavy model", Some(&model))?;
        (fast, medium, heavy)
    } else {
        (model.clone(), model.clone(), model)
    };

    Ok(Some(provider_assignments(
        ProviderSetup {
            provider: "openai-compatible",
            base_url,
            api_key,
            fast,
            medium,
            heavy,
            disable_fast_thinking_default: None,
        },
        existing,
    )))
}

fn prompt_api_key(
    input: &mut impl BufRead,
    output: &mut impl Write,
    existing: &HashMap<String, String>,
) -> io::Result<String> {
    let existing_key = existing
        .get("LLM_API_KEY")
        .filter(|key| !is_placeholder_api_key(key));

    if existing_key.is_some() {
        let value = prompt_line(
            input,
            output,
            "API key [press Enter to keep existing]",
            None,
        )?;
        if value.is_empty() {
            return Ok(existing_key.cloned().unwrap_or_default());
        }
        return Ok(value);
    }

    loop {
        writeln!(output, "API key input is visible in this terminal.")?;
        let value = prompt_line(input, output, "API key", None)?;
        if !value.is_empty() && !is_placeholder_api_key(&value) {
            return Ok(value);
        }
        writeln!(
            output,
            "Please enter a real API key, or choose mock provider instead."
        )?;
    }
}

fn prompt_model(
    input: &mut impl BufRead,
    output: &mut impl Write,
    existing: &HashMap<String, String>,
) -> io::Result<String> {
    prompt_model_with_default(input, output, existing, DEFAULT_MODEL)
}

fn prompt_model_with_default(
    input: &mut impl BufRead,
    output: &mut impl Write,
    existing: &HashMap<String, String>,
    default: &str,
) -> io::Result<String> {
    let default_model = existing
        .get("LLM_MODEL")
        .or_else(|| existing.get("LLM_MODEL_FAST"))
        .map(String::as_str)
        .unwrap_or(default);
    prompt_line(input, output, "Model", Some(default_model))
}

fn provider_assignments(
    setup: ProviderSetup,
    existing: &HashMap<String, String>,
) -> Vec<EnvAssignment> {
    vec![
        assignment("LLM_PROVIDER", setup.provider),
        assignment("LLM_BASE_URL", setup.base_url),
        assignment("LLM_API_KEY", setup.api_key),
        assignment("LLM_MODEL", setup.heavy.clone()),
        assignment("LLM_MODEL_FAST", setup.fast),
        assignment("LLM_MODEL_MEDIUM", setup.medium),
        assignment("LLM_MODEL_HEAVY", setup.heavy),
        assignment(
            "LLM_MAX_TOKENS",
            existing_or_default(existing, "LLM_MAX_TOKENS", DEFAULT_MAX_TOKENS),
        ),
        assignment(
            "LLM_MAX_TOKENS_FAST",
            existing_or_default(existing, "LLM_MAX_TOKENS_FAST", DEFAULT_MAX_TOKENS_FAST),
        ),
        assignment(
            "LLM_MAX_TOKENS_MEDIUM",
            existing_or_default(existing, "LLM_MAX_TOKENS_MEDIUM", ""),
        ),
        assignment(
            "LLM_MAX_TOKENS_HEAVY",
            existing_or_default(existing, "LLM_MAX_TOKENS_HEAVY", ""),
        ),
        assignment(
            "LLM_TEMPERATURE",
            existing_or_default(existing, "LLM_TEMPERATURE", DEFAULT_TEMPERATURE),
        ),
        assignment(
            "LLM_DISABLE_FAST_THINKING",
            existing_or_default(
                existing,
                "LLM_DISABLE_FAST_THINKING",
                setup.disable_fast_thinking_default.unwrap_or(""),
            ),
        ),
    ]
}

fn mock_assignments(existing: &HashMap<String, String>) -> Vec<EnvAssignment> {
    vec![
        assignment("LLM_PROVIDER", "mock"),
        assignment(
            "LLM_BASE_URL",
            existing_or_default(existing, "LLM_BASE_URL", DEFAULT_OPENAI_BASE_URL),
        ),
        assignment(
            "LLM_API_KEY",
            existing_or_default(existing, "LLM_API_KEY", PLACEHOLDER_API_KEY),
        ),
        assignment(
            "LLM_MODEL",
            existing_or_default(existing, "LLM_MODEL", DEFAULT_MODEL),
        ),
        assignment(
            "LLM_MODEL_FAST",
            existing_or_default(existing, "LLM_MODEL_FAST", DEFAULT_MODEL),
        ),
        assignment(
            "LLM_MODEL_MEDIUM",
            existing_or_default(existing, "LLM_MODEL_MEDIUM", ""),
        ),
        assignment(
            "LLM_MODEL_HEAVY",
            existing_or_default(existing, "LLM_MODEL_HEAVY", ""),
        ),
        assignment(
            "LLM_MAX_TOKENS",
            existing_or_default(existing, "LLM_MAX_TOKENS", DEFAULT_MAX_TOKENS),
        ),
        assignment(
            "LLM_MAX_TOKENS_FAST",
            existing_or_default(existing, "LLM_MAX_TOKENS_FAST", DEFAULT_MAX_TOKENS_FAST),
        ),
        assignment(
            "LLM_MAX_TOKENS_MEDIUM",
            existing_or_default(existing, "LLM_MAX_TOKENS_MEDIUM", ""),
        ),
        assignment(
            "LLM_MAX_TOKENS_HEAVY",
            existing_or_default(existing, "LLM_MAX_TOKENS_HEAVY", ""),
        ),
        assignment(
            "LLM_TEMPERATURE",
            existing_or_default(existing, "LLM_TEMPERATURE", DEFAULT_TEMPERATURE),
        ),
        assignment(
            "LLM_DISABLE_FAST_THINKING",
            existing_or_default(existing, "LLM_DISABLE_FAST_THINKING", ""),
        ),
    ]
}

fn prompt_yes_no(
    input: &mut impl BufRead,
    output: &mut impl Write,
    label: &str,
    default_yes: bool,
) -> io::Result<bool> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    loop {
        let value = prompt_line(input, output, &format!("{label} {suffix}"), None)?;
        if value.is_empty() {
            return Ok(default_yes);
        }
        match value.to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => writeln!(output, "Please answer y or n.")?,
        }
    }
}

fn prompt_line(
    input: &mut impl BufRead,
    output: &mut impl Write,
    label: &str,
    default: Option<&str>,
) -> io::Result<String> {
    match default {
        Some(default) => write!(output, "{label} [{default}]: ")?,
        None => write!(output, "{label}: ")?,
    }
    output.flush()?;

    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return default.map(ToOwned::to_owned).ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "setup input ended early")
        });
    }
    let value = line.trim().to_string();
    if value.is_empty() {
        Ok(default.unwrap_or_default().to_string())
    } else {
        Ok(value)
    }
}

fn env_file_needs_setup(path: &Path) -> bool {
    if !path.exists() {
        return true;
    }

    let values = read_env_values(path);
    match provider_value(&values) {
        Some("mock") => false,
        Some("pollinations-free") => missing_value(&values, "LLM_BASE_URL"),
        Some("openai-compatible" | "anthropic") => {
            missing_value(&values, "LLM_BASE_URL")
                || values
                    .get("LLM_API_KEY")
                    .is_none_or(|value| missing_or_placeholder(value))
        }
        Some(_) => true,
        None => true,
    }
}

fn provider_value(values: &HashMap<String, String>) -> Option<&str> {
    values.get("LLM_PROVIDER").map(String::as_str)
}

fn read_env_values(path: &Path) -> HashMap<String, String> {
    fs::read_to_string(path)
        .map(|content| parse_env_values(&content))
        .unwrap_or_default()
}

fn parse_env_values(content: &str) -> HashMap<String, String> {
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

fn parse_env_value(value: &str) -> String {
    if let Some(quoted) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        quoted.replace("\\\"", "\"").replace("\\\\", "\\")
    } else if let Some(quoted) = value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')) {
        quoted.to_string()
    } else {
        value.to_string()
    }
}

fn write_env_assignments(path: &Path, assignments: &[EnvAssignment]) -> io::Result<()> {
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

    for key in LLM_ENV_KEYS {
        if let Some(value) = assignments_by_key.get(key)
            && !seen.contains(*key)
        {
            lines.push(format!("{key}={}", format_env_value(value)));
        }
    }

    let mut content = lines.join("\n");
    content.push('\n');

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

fn env_line_key(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return None;
    }
    let (key, _) = trimmed.split_once('=')?;
    let key = key.trim();
    (!key.is_empty()).then_some(key)
}

fn format_env_value(value: &str) -> String {
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

fn existing_or_default(
    existing: &HashMap<String, String>,
    key: &str,
    default_value: &str,
) -> String {
    existing
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| default_value.to_string())
}

fn assignment(key: &'static str, value: impl Into<String>) -> EnvAssignment {
    EnvAssignment {
        key,
        value: value.into(),
    }
}

fn missing_value(values: &HashMap<String, String>, key: &str) -> bool {
    values.get(key).is_none_or(|value| value.trim().is_empty())
}

fn missing_or_placeholder(value: &str) -> bool {
    value.trim().is_empty() || is_placeholder_api_key(value)
}

fn is_placeholder_api_key(value: &str) -> bool {
    value.trim() == PLACEHOLDER_API_KEY
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn missing_env_file_needs_setup() {
        let dir = tempfile::tempdir().unwrap();
        assert!(env_file_needs_setup(&dir.path().join(".env")));
    }

    #[test]
    fn mock_provider_is_considered_configured() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(&path, "LLM_PROVIDER=mock\n").unwrap();

        assert!(!env_file_needs_setup(&path));
    }

    #[test]
    fn placeholder_openai_key_needs_setup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(
            &path,
            "LLM_PROVIDER=openai-compatible\nLLM_BASE_URL=https://api.openai.com/v1\nLLM_API_KEY=sk-your-key-here\n",
        )
        .unwrap();

        assert!(env_file_needs_setup(&path));
    }

    #[test]
    fn complete_openai_config_does_not_need_setup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(
            &path,
            "LLM_PROVIDER=openai-compatible\nLLM_BASE_URL=https://api.openai.com/v1\nLLM_API_KEY=sk-real\n",
        )
        .unwrap();

        assert!(!env_file_needs_setup(&path));
    }

    #[test]
    fn complete_pollinations_free_config_does_not_need_setup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(
            &path,
            "LLM_PROVIDER=pollinations-free\nLLM_BASE_URL=https://text.pollinations.ai/openai\nLLM_API_KEY=\n",
        )
        .unwrap();

        assert!(!env_file_needs_setup(&path));
    }

    #[test]
    fn setup_wizard_writes_pollinations_free_config_without_api_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        let mut input = Cursor::new("y\n1\n\n");
        let mut output = Vec::new();

        let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

        assert!(saved);
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("LLM_PROVIDER=pollinations-free"));
        assert!(content.contains("LLM_BASE_URL=https://text.pollinations.ai/openai"));
        assert!(content.contains("LLM_API_KEY=\n"));
        assert!(content.contains("LLM_MODEL=openai-fast"));
    }

    #[test]
    fn complete_anthropic_config_does_not_need_setup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(
            &path,
            "LLM_PROVIDER=anthropic\nLLM_BASE_URL=https://api.anthropic.com\nLLM_API_KEY=sk-ant-real\n",
        )
        .unwrap();

        assert!(!env_file_needs_setup(&path));
    }

    #[test]
    fn setup_wizard_writes_openai_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        let mut input = Cursor::new("y\n2\nsk-real\ncustom-model\n");
        let mut output = Vec::new();

        let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

        assert!(saved);
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("LLM_PROVIDER=openai-compatible"));
        assert!(content.contains("LLM_BASE_URL=https://api.openai.com/v1"));
        assert!(content.contains("LLM_API_KEY=sk-real"));
        assert!(content.contains("LLM_MODEL=custom-model"));
        assert!(content.contains("LLM_MODEL_FAST=custom-model"));
        assert!(content.contains("LLM_MODEL_MEDIUM=custom-model"));
        assert!(content.contains("LLM_MODEL_HEAVY=custom-model"));
    }

    #[test]
    fn setup_wizard_writes_anthropic_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        let mut input = Cursor::new("y\n3\nsk-ant-real\n\n");
        let mut output = Vec::new();

        let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

        assert!(saved);
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("LLM_PROVIDER=anthropic"));
        assert!(content.contains("LLM_BASE_URL=https://api.anthropic.com"));
        assert!(content.contains("LLM_API_KEY=sk-ant-real"));
        assert!(content.contains("LLM_MODEL=claude-sonnet-4-6"));
    }

    #[test]
    fn setup_wizard_writes_deepseek_fast_thinking_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        let mut input = Cursor::new("y\n5\nsk-deepseek\n\n");
        let mut output = Vec::new();

        let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

        assert!(saved);
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("LLM_BASE_URL=https://api.deepseek.com"));
        assert!(content.contains("LLM_MODEL=deepseek-v4-flash"));
        assert!(content.contains("LLM_DISABLE_FAST_THINKING=true"));
    }

    #[test]
    fn setup_wizard_writes_openrouter_free_router_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        let mut input = Cursor::new("y\n7\nsk-or\n\n");
        let mut output = Vec::new();

        let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

        assert!(saved);
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("LLM_BASE_URL=https://openrouter.ai/api/v1"));
        assert!(content.contains("LLM_MODEL=openrouter/free"));
    }

    #[test]
    fn setup_wizard_can_write_compatible_tiered_models() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        let mut input = Cursor::new(
            "y\n10\nhttps://api.example.com/v1\nsk-real\nfallback\ny\nfast\nmedium\nheavy\n",
        );
        let mut output = Vec::new();

        let saved = run_api_setup_with_io(&path, &mut input, &mut output, false).unwrap();

        assert!(saved);
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("LLM_BASE_URL=https://api.example.com/v1"));
        assert!(content.contains("LLM_MODEL_FAST=fast"));
        assert!(content.contains("LLM_MODEL_MEDIUM=medium"));
        assert!(content.contains("LLM_MODEL_HEAVY=heavy"));
    }

    #[test]
    fn env_writer_preserves_unrelated_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(&path, "# keep me\nOTHER=value\nLLM_PROVIDER=mock\n").unwrap();

        write_env_assignments(&path, &[assignment("LLM_PROVIDER", "openai-compatible")]).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("# keep me"));
        assert!(content.contains("OTHER=value"));
        assert!(content.contains("LLM_PROVIDER=openai-compatible"));
    }

    // ====================================================================
    // Additional unit tests for helper functions
    // ====================================================================

    // --- parse_env_values ---

    #[test]
    fn parse_env_values_key_value() {
        let result = parse_env_values("KEY=value\nOTHER=other\n");
        assert_eq!(result.get("KEY").map(String::as_str), Some("value"));
        assert_eq!(result.get("OTHER").map(String::as_str), Some("other"));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parse_env_values_with_spaces() {
        let result = parse_env_values("  KEY = value with spaces  \n");
        assert_eq!(
            result.get("KEY").map(String::as_str),
            Some("value with spaces")
        );
    }

    #[test]
    fn parse_env_values_comments_and_blanks() {
        let result = parse_env_values("# comment\n\n  # indented\n\nKEY=val\n");
        assert_eq!(result.get("KEY").map(String::as_str), Some("val"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn parse_env_values_double_quoted() {
        let result = parse_env_values("KEY=\"hello world\"\n");
        assert_eq!(result.get("KEY").map(String::as_str), Some("hello world"));
    }

    #[test]
    fn parse_env_values_single_quoted() {
        let result = parse_env_values("KEY='single quoted'\n");
        assert_eq!(result.get("KEY").map(String::as_str), Some("single quoted"));
    }

    #[test]
    fn parse_env_values_escaped_quote() {
        let content = "KEY=\"escaped \\\"quote\\\"\"\n";
        let result = parse_env_values(content);
        assert_eq!(
            result.get("KEY").map(String::as_str),
            Some("escaped \"quote\"")
        );
    }

    #[test]
    fn parse_env_values_backslash_escapes() {
        let content = "KEY=\"path\\\\to\\\\file\"\n";
        let result = parse_env_values(content);
        assert_eq!(
            result.get("KEY").map(String::as_str),
            Some("path\\to\\file")
        );
    }

    #[test]
    fn parse_env_values_no_equals_skipped() {
        let result = parse_env_values("JUSTTEXT\nKEY=val\n");
        assert_eq!(result.get("KEY").map(String::as_str), Some("val"));
        assert!(result.get("JUSTTEXT").is_none());
    }

    #[test]
    fn parse_env_values_empty_key_skipped() {
        let result = parse_env_values("=value\nKEY=real\n");
        assert_eq!(result.get("KEY").map(String::as_str), Some("real"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn parse_env_values_empty_value() {
        let result = parse_env_values("KEY=\n");
        assert_eq!(result.get("KEY").map(String::as_str), Some(""));
    }

    // --- parse_env_value ---

    #[test]
    fn parse_env_value_unquoted() {
        assert_eq!(parse_env_value("hello"), "hello");
        assert_eq!(parse_env_value("hello world"), "hello world");
        assert_eq!(parse_env_value(""), "");
    }

    #[test]
    fn parse_env_value_double_quoted_basic() {
        assert_eq!(parse_env_value("\"hello\""), "hello");
        assert_eq!(parse_env_value("\"hello world\""), "hello world");
    }

    #[test]
    fn parse_env_value_double_quoted_escapes() {
        assert_eq!(
            parse_env_value("\"hello \\\"world\\\"\""),
            "hello \"world\""
        );
        assert_eq!(parse_env_value("\"path\\\\to\""), "path\\to");
        assert_eq!(parse_env_value("\"\\\\\\\"\""), "\\\"");
    }

    #[test]
    fn parse_env_value_single_quoted_basic() {
        assert_eq!(parse_env_value("'hello'"), "hello");
        assert_eq!(parse_env_value("'hello world'"), "hello world");
    }

    #[test]
    fn parse_env_value_single_quoted_no_escapes() {
        assert_eq!(parse_env_value("'back\\slash'"), "back\\slash");
        assert_eq!(parse_env_value("'quote\"inside'"), "quote\"inside");
    }

    // --- env_line_key ---

    #[test]
    fn env_line_key_simple() {
        assert_eq!(env_line_key("KEY=value"), Some("KEY"));
        assert_eq!(env_line_key("LLM_PROVIDER=mock"), Some("LLM_PROVIDER"));
    }

    #[test]
    fn env_line_key_with_whitespace() {
        assert_eq!(env_line_key("  KEY = value"), Some("KEY"));
        assert_eq!(env_line_key("\tKEY=value"), Some("KEY"));
    }

    #[test]
    fn env_line_key_comment_returns_none() {
        assert_eq!(env_line_key("# comment"), None);
        assert_eq!(env_line_key("  # indented"), None);
    }

    #[test]
    fn env_line_key_no_equals() {
        assert_eq!(env_line_key("JUSTTEXT"), None);
        assert_eq!(env_line_key(""), None);
    }

    #[test]
    fn env_line_key_empty_key() {
        assert_eq!(env_line_key("=value"), None);
        assert_eq!(env_line_key("  =value"), None);
    }

    // --- format_env_value ---

    #[test]
    fn format_env_value_empty() {
        assert_eq!(format_env_value(""), "");
    }

    #[test]
    fn format_env_value_safe_chars_no_quoting() {
        assert_eq!(format_env_value("gpt-4o-mini"), "gpt-4o-mini");
        assert_eq!(
            format_env_value("https://api.openai.com/v1"),
            "https://api.openai.com/v1"
        );
        assert_eq!(format_env_value("openai-fast"), "openai-fast");
        assert_eq!(format_env_value("0.7"), "0.7");
        assert_eq!(format_env_value("true"), "true");
    }

    #[test]
    fn format_env_value_needs_quoting() {
        assert_eq!(format_env_value("hello world"), "\"hello world\"");
        assert_eq!(format_env_value("key=value"), "\"key=value\"");
        assert_eq!(format_env_value("a@b"), "\"a@b\"");
    }

    #[test]
    fn format_env_value_escapes_special_chars() {
        assert_eq!(format_env_value("say \"hi\""), "\"say \\\"hi\\\"\"");
        assert_eq!(format_env_value("a\\b"), "\"a\\\\b\"");
    }

    // --- existing_or_default ---

    #[test]
    fn existing_or_default_key_present() {
        let mut map = HashMap::new();
        map.insert("LLM_MODEL".to_string(), "gpt-5".to_string());
        assert_eq!(existing_or_default(&map, "LLM_MODEL", "fallback"), "gpt-5");
    }

    #[test]
    fn existing_or_default_key_missing() {
        let map: HashMap<String, String> = HashMap::new();
        assert_eq!(
            existing_or_default(&map, "LLM_MODEL", "gpt-4o-mini"),
            "gpt-4o-mini"
        );
    }

    #[test]
    fn existing_or_default_empty_value_falls_back() {
        let mut map = HashMap::new();
        map.insert("KEY".to_string(), "".to_string());
        assert_eq!(existing_or_default(&map, "KEY", "default"), "default");
    }

    #[test]
    fn existing_or_default_whitespace_value_falls_back() {
        let mut map = HashMap::new();
        map.insert("KEY".to_string(), "   ".to_string());
        assert_eq!(existing_or_default(&map, "KEY", "default"), "default");
    }

    // --- assignment ---

    #[test]
    fn assignment_creates_correct_struct() {
        let a = assignment("LLM_PROVIDER", "openai-compatible");
        assert_eq!(a.key, "LLM_PROVIDER");
        assert_eq!(a.value, "openai-compatible");
    }

    #[test]
    fn assignment_accepts_into_string() {
        let a = assignment("KEY", String::from("value"));
        assert_eq!(a.key, "KEY");
        assert_eq!(a.value, "value");
    }

    // --- missing_value ---

    #[test]
    fn missing_value_when_key_missing() {
        let values = HashMap::new();
        assert!(missing_value(&values, "LLM_API_KEY"));
    }

    #[test]
    fn missing_value_when_empty_or_whitespace() {
        let mut values = HashMap::new();
        values.insert("LLM_API_KEY".to_string(), "".to_string());
        assert!(missing_value(&values, "LLM_API_KEY"));
        values.insert("LLM_API_KEY".to_string(), "  ".to_string());
        assert!(missing_value(&values, "LLM_API_KEY"));
    }

    #[test]
    fn missing_value_when_present() {
        let mut values = HashMap::new();
        values.insert("LLM_API_KEY".to_string(), "sk-real".to_string());
        assert!(!missing_value(&values, "LLM_API_KEY"));
    }

    // --- missing_or_placeholder ---

    #[test]
    fn missing_or_placeholder_empty_or_whitespace() {
        assert!(missing_or_placeholder(""));
        assert!(missing_or_placeholder("   "));
    }

    #[test]
    fn missing_or_placeholder_is_placeholder() {
        assert!(missing_or_placeholder("sk-your-key-here"));
        assert!(missing_or_placeholder("  sk-your-key-here  "));
    }

    #[test]
    fn missing_or_placeholder_real_value() {
        assert!(!missing_or_placeholder("sk-real-key-12345"));
    }

    // --- is_placeholder_api_key ---

    #[test]
    fn is_placeholder_api_key_matches() {
        assert!(is_placeholder_api_key("sk-your-key-here"));
        assert!(is_placeholder_api_key("  sk-your-key-here  "));
    }

    #[test]
    fn is_placeholder_api_key_does_not_match() {
        assert!(!is_placeholder_api_key("sk-real"));
        assert!(!is_placeholder_api_key(""));
        assert!(!is_placeholder_api_key("sk-your-key-here-real"));
    }

    // --- provider_value ---

    #[test]
    fn provider_value_present() {
        let mut values = HashMap::new();
        values.insert("LLM_PROVIDER".to_string(), "openai-compatible".to_string());
        assert_eq!(provider_value(&values), Some("openai-compatible"));
    }

    #[test]
    fn provider_value_missing() {
        let values: HashMap<String, String> = HashMap::new();
        assert_eq!(provider_value(&values), None);
    }

    // --- read_env_values ---

    #[test]
    fn read_env_values_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(&path, "LLM_PROVIDER=mock\nLLM_MODEL=gpt-5\n").unwrap();
        let values = read_env_values(&path);
        assert_eq!(values.get("LLM_PROVIDER").map(String::as_str), Some("mock"));
        assert_eq!(values.get("LLM_MODEL").map(String::as_str), Some("gpt-5"));
    }

    #[test]
    fn read_env_values_file_missing_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env.nonexistent");
        let values = read_env_values(&path);
        assert!(values.is_empty());
    }

    // --- env_file_needs_setup more edge cases ---

    #[test]
    fn env_file_needs_setup_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(&path, "").unwrap();
        assert!(env_file_needs_setup(&path));
    }

    #[test]
    fn env_file_needs_setup_unknown_provider() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(&path, "LLM_PROVIDER=some-unknown\n").unwrap();
        assert!(env_file_needs_setup(&path));
    }

    #[test]
    fn env_file_needs_setup_anthropic_missing_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(
            &path,
            "LLM_PROVIDER=anthropic\nLLM_BASE_URL=https://api.anthropic.com\nLLM_API_KEY=\n",
        )
        .unwrap();
        assert!(env_file_needs_setup(&path));
    }

    #[test]
    fn env_file_needs_setup_anthropic_placeholder_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(
            &path,
            "LLM_PROVIDER=anthropic\nLLM_BASE_URL=https://api.anthropic.com\nLLM_API_KEY=sk-your-key-here\n",
        )
        .unwrap();
        assert!(env_file_needs_setup(&path));
    }

    #[test]
    fn env_file_needs_setup_pollinations_missing_base_url() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(&path, "LLM_PROVIDER=pollinations-free\n").unwrap();
        assert!(env_file_needs_setup(&path));
    }

    #[test]
    fn env_file_needs_setup_no_provider_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(&path, "LLM_MODEL=gpt-5\n").unwrap();
        assert!(env_file_needs_setup(&path));
    }

    // --- provider_assignments ---

    #[test]
    fn provider_assignments_has_all_13_keys() {
        let setup = ProviderSetup {
            provider: "test-provider",
            base_url: "https://test.example.com".to_string(),
            api_key: "sk-test".to_string(),
            fast: "fast-model".to_string(),
            medium: "medium-model".to_string(),
            heavy: "heavy-model".to_string(),
            disable_fast_thinking_default: Some("true"),
        };
        let existing = HashMap::new();
        let assignments = provider_assignments(setup, &existing);
        assert_eq!(assignments.len(), 13);

        let keys: Vec<&str> = assignments.iter().map(|a| a.key).collect();
        for expected_key in LLM_ENV_KEYS {
            assert!(keys.contains(expected_key), "missing key: {expected_key}");
        }

        assert_eq!(find_value(&assignments, "LLM_PROVIDER"), "test-provider");
        assert_eq!(
            find_value(&assignments, "LLM_BASE_URL"),
            "https://test.example.com"
        );
        assert_eq!(find_value(&assignments, "LLM_API_KEY"), "sk-test");
        assert_eq!(find_value(&assignments, "LLM_MODEL"), "heavy-model");
        assert_eq!(find_value(&assignments, "LLM_MODEL_FAST"), "fast-model");
        assert_eq!(find_value(&assignments, "LLM_MODEL_MEDIUM"), "medium-model");
        assert_eq!(find_value(&assignments, "LLM_MODEL_HEAVY"), "heavy-model");
        assert_eq!(
            find_value(&assignments, "LLM_MAX_TOKENS"),
            DEFAULT_MAX_TOKENS
        );
        assert_eq!(
            find_value(&assignments, "LLM_MAX_TOKENS_FAST"),
            DEFAULT_MAX_TOKENS_FAST
        );
        assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS_MEDIUM"), "");
        assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS_HEAVY"), "");
        assert_eq!(
            find_value(&assignments, "LLM_TEMPERATURE"),
            DEFAULT_TEMPERATURE
        );
        assert_eq!(
            find_value(&assignments, "LLM_DISABLE_FAST_THINKING"),
            "true"
        );
    }

    #[test]
    fn provider_assignments_respects_existing_values() {
        let setup = ProviderSetup {
            provider: "test-provider",
            base_url: "https://test.example.com".to_string(),
            api_key: "sk-test".to_string(),
            fast: "fast".to_string(),
            medium: "medium".to_string(),
            heavy: "heavy".to_string(),
            disable_fast_thinking_default: None,
        };
        let mut existing = HashMap::new();
        existing.insert("LLM_MAX_TOKENS".to_string(), "999".to_string());
        existing.insert("LLM_MAX_TOKENS_HEAVY".to_string(), "888".to_string());
        existing.insert("LLM_TEMPERATURE".to_string(), "0.3".to_string());

        let assignments = provider_assignments(setup, &existing);
        assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS"), "999");
        assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS_HEAVY"), "888");
        assert_eq!(find_value(&assignments, "LLM_TEMPERATURE"), "0.3");
    }

    #[test]
    fn provider_assignments_disable_fast_thinking_none() {
        let setup = ProviderSetup {
            provider: "test-provider",
            base_url: "https://test.example.com".to_string(),
            api_key: "sk-test".to_string(),
            fast: "fast".to_string(),
            medium: "medium".to_string(),
            heavy: "heavy".to_string(),
            disable_fast_thinking_default: None,
        };
        let existing = HashMap::new();
        let assignments = provider_assignments(setup, &existing);
        assert_eq!(find_value(&assignments, "LLM_DISABLE_FAST_THINKING"), "");
    }

    // --- mock_assignments ---

    #[test]
    fn mock_assignments_has_all_13_keys() {
        let existing = HashMap::new();
        let assignments = mock_assignments(&existing);
        assert_eq!(assignments.len(), 13);

        let keys: Vec<&str> = assignments.iter().map(|a| a.key).collect();
        for expected_key in LLM_ENV_KEYS {
            assert!(keys.contains(expected_key), "missing key: {expected_key}");
        }

        assert_eq!(find_value(&assignments, "LLM_PROVIDER"), "mock");
        assert_eq!(
            find_value(&assignments, "LLM_BASE_URL"),
            DEFAULT_OPENAI_BASE_URL
        );
        assert_eq!(find_value(&assignments, "LLM_API_KEY"), PLACEHOLDER_API_KEY);
        assert_eq!(find_value(&assignments, "LLM_MODEL"), DEFAULT_MODEL);
        assert_eq!(find_value(&assignments, "LLM_MODEL_FAST"), DEFAULT_MODEL);
        assert_eq!(find_value(&assignments, "LLM_MODEL_MEDIUM"), "");
        assert_eq!(find_value(&assignments, "LLM_MODEL_HEAVY"), "");
        assert_eq!(
            find_value(&assignments, "LLM_MAX_TOKENS"),
            DEFAULT_MAX_TOKENS
        );
        assert_eq!(
            find_value(&assignments, "LLM_MAX_TOKENS_FAST"),
            DEFAULT_MAX_TOKENS_FAST
        );
        assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS_MEDIUM"), "");
        assert_eq!(find_value(&assignments, "LLM_MAX_TOKENS_HEAVY"), "");
        assert_eq!(
            find_value(&assignments, "LLM_TEMPERATURE"),
            DEFAULT_TEMPERATURE
        );
        assert_eq!(find_value(&assignments, "LLM_DISABLE_FAST_THINKING"), "");
    }

    #[test]
    fn mock_assignments_respects_existing_values() {
        let mut existing = HashMap::new();
        existing.insert("LLM_MODEL".to_string(), "custom-model".to_string());
        existing.insert(
            "LLM_BASE_URL".to_string(),
            "https://custom.example.com".to_string(),
        );
        existing.insert("LLM_API_KEY".to_string(), "sk-custom".to_string());

        let assignments = mock_assignments(&existing);
        assert_eq!(find_value(&assignments, "LLM_MODEL"), "custom-model");
        assert_eq!(
            find_value(&assignments, "LLM_BASE_URL"),
            "https://custom.example.com"
        );
        assert_eq!(find_value(&assignments, "LLM_API_KEY"), "sk-custom");
    }

    // --- EnvAssignment struct ---

    #[test]
    fn env_assignment_construction() {
        let a = EnvAssignment {
            key: "TEST",
            value: "val".to_string(),
        };
        assert_eq!(a.key, "TEST");
        assert_eq!(a.value, "val");
    }

    #[test]
    fn env_assignment_equality() {
        let a = EnvAssignment {
            key: "KEY",
            value: "val".to_string(),
        };
        let b = EnvAssignment {
            key: "KEY",
            value: "val".to_string(),
        };
        let c = EnvAssignment {
            key: "KEY",
            value: "other".to_string(),
        };
        let d = EnvAssignment {
            key: "OTHER",
            value: "val".to_string(),
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn env_assignment_debug_format() {
        let a = assignment("LLM_PROVIDER", "mock");
        let dbg = format!("{a:?}");
        assert!(dbg.contains("LLM_PROVIDER"));
        assert!(dbg.contains("mock"));
    }

    // --- ApiPreset ---

    #[test]
    fn all_api_presets_have_required_fields() {
        assert!(!API_PRESETS.is_empty());
        for (i, preset) in API_PRESETS.iter().enumerate() {
            assert!(!preset.name.is_empty(), "preset {i}: name empty");
            assert!(
                !preset.description.is_empty(),
                "preset {i}: description empty"
            );
            assert!(!preset.provider.is_empty(), "preset {i}: provider empty");
            assert!(!preset.base_url.is_empty(), "preset {i}: base_url empty");
            assert!(!preset.model.is_empty(), "preset {i}: model empty");
        }
    }

    #[test]
    fn pollinations_free_preset_no_api_key() {
        let preset = &API_PRESETS[0];
        assert_eq!(preset.name, "Pollinations Free");
        assert!(!preset.requires_api_key);
        assert_eq!(preset.disable_fast_thinking, "");
    }

    #[test]
    fn deepseek_preset_disables_fast_thinking() {
        let preset = API_PRESETS
            .iter()
            .find(|p| p.name == "DeepSeek")
            .expect("DeepSeek preset not found");
        assert_eq!(preset.disable_fast_thinking, "true");
        assert_eq!(preset.provider, "openai-compatible");
    }

    #[test]
    fn all_openai_compatible_presets_require_key() {
        for preset in API_PRESETS
            .iter()
            .filter(|p| p.provider == "openai-compatible")
        {
            assert!(preset.requires_api_key);
        }
    }

    #[test]
    fn api_presets_count_is_9() {
        assert_eq!(API_PRESETS.len(), 9);
    }

    // --- helper ---

    fn find_value(assignments: &[EnvAssignment], key: &str) -> String {
        assignments
            .iter()
            .find(|a| a.key == key)
            .map(|a| a.value.clone())
            .unwrap_or_default()
    }
}
