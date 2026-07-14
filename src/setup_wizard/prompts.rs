use std::collections::HashMap;
use std::io::{self, BufRead, IsTerminal, Write};

use super::env_file::is_placeholder_api_key;
use super::types::DEFAULT_MODEL;

pub(crate) fn prompt_api_key(
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
        let value = if std::io::stdin().is_terminal() {
            write!(output, "API key (input hidden): ")?;
            output.flush()?;
            match rpassword::read_password() {
                Ok(pw) => pw,
                Err(_) => {
                    writeln!(
                        output,
                        "\nFailed to hide input, falling back to visible mode."
                    )?;
                    prompt_line(input, output, "API key", None)?
                }
            }
        } else {
            writeln!(output, "API key input is visible in this terminal.")?;
            prompt_line(input, output, "API key", None)?
        };
        if !value.is_empty() && !is_placeholder_api_key(&value) {
            return Ok(value);
        }
        writeln!(
            output,
            "Please enter a real API key, or choose mock provider instead."
        )?;
    }
}

pub(crate) fn prompt_model(
    input: &mut impl BufRead,
    output: &mut impl Write,
    existing: &HashMap<String, String>,
) -> io::Result<String> {
    prompt_model_with_default(input, output, existing, DEFAULT_MODEL)
}

pub(crate) fn prompt_model_with_default(
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

pub(crate) fn prompt_yes_no(
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

pub(crate) fn prompt_line(
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
