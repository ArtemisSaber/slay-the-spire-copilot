use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use super::assignments::{mock_assignments, provider_assignments};
use super::env_file::{env_file_needs_setup, read_env_values, write_env_assignments};
use super::prompts::{
    prompt_api_key, prompt_line, prompt_model, prompt_model_with_default, prompt_yes_no,
};
use super::types::{
    API_PRESETS, ApiPreset, DEFAULT_OPENAI_BASE_URL, ENV_FILE_NAME, EnvAssignment, ProviderSetup,
};

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

pub(crate) fn run_api_setup_with_io(
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

        let default_choice = if super::env_file::provider_value(existing) == Some("mock") {
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
