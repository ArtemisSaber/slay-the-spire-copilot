use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

mod aggregation;
mod render;

pub const POSTMORTEM_FILE_NAME: &str = "postmortem.md";

/// Minimum trimmed length (in chars) for an AI postmortem report to be
/// considered valid. Below this the LLM output is treated as garbage and
/// the deterministic report is used alone.
const MIN_AI_POSTMORTEM_LEN: usize = 50;

fn is_valid_ai_report(ai_report: &str) -> bool {
    let trimmed = ai_report.trim();
    !trimmed.is_empty() && trimmed.chars().count() >= MIN_AI_POSTMORTEM_LEN
}

pub fn postmortem_path_for_journal(journal_path: &Path) -> PathBuf {
    journal_path
        .parent()
        .map(|dir| dir.join(POSTMORTEM_FILE_NAME))
        .unwrap_or_else(|| PathBuf::from(POSTMORTEM_FILE_NAME))
}

pub fn generate_report_from_journal_file(
    journal_path: &Path,
    locale: &crate::locales::Locale,
) -> Result<String, String> {
    let content = fs::read_to_string(journal_path).map_err(|e| e.to_string())?;
    generate_report_from_jsonl(&content, locale)
}

pub fn combine_postmortem_report(
    ai_report: &str,
    deterministic_report: &str,
    section_machine: &str,
) -> String {
    if !is_valid_ai_report(ai_report) {
        tracing::warn!(
            "AI postmortem report empty or too short ({} chars), using deterministic report only",
            ai_report.trim().chars().count()
        );
        return format!("{section_machine}\n\n{deterministic_report}");
    }
    format!("{ai_report}\n\n---\n\n{section_machine}\n\n{deterministic_report}")
}

pub fn write_report_for_journal(journal_path: &Path, report: &str) -> Result<PathBuf, String> {
    let report_path = postmortem_path_for_journal(journal_path);
    atomic_write(&report_path, report).map_err(|e| e.to_string())?;
    Ok(report_path)
}

fn atomic_write(path: &Path, content: &str) -> Result<(), std::io::Error> {
    let tmp = tmp_path(path);
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    let mut name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    name.push_str(".tmp");
    tmp.set_file_name(name);
    tmp
}

pub fn generate_report_from_jsonl(
    input: &str,
    locale: &crate::locales::Locale,
) -> Result<String, String> {
    let summary = aggregation::collect_journal(input, &locale.postmortem)?;
    Ok(render::render_report(&summary, &locale.postmortem))
}

pub fn build_ai_postmortem_prompt(
    deterministic_report: &str,
    locale: &crate::locales::Locale,
    outcome: &str,
) -> String {
    let pm = &locale.postmortem;
    let lang_name = &locale.language_name;
    format!(
        "{}\n\n{}\n{}\n{}\n{}\n{}\n\n{}\n\nOutcome: {outcome}\n\n{deterministic_report}\n",
        pm.ai_prompt.replace("{lang_name}", lang_name),
        pm.ai_requirements,
        pm.ai_req1,
        pm.ai_req2,
        pm.ai_req3,
        pm.ai_req4,
        pm.machine_summary,
    )
}

#[derive(Debug)]
struct RewardSnapshot {
    choices: Vec<(String, String)>,
    deck_counts: HashMap<String, usize>,
}

impl RewardSnapshot {
    fn from_state(state: &Value) -> Option<Self> {
        let choices = state
            .get("card_reward_choices")?
            .as_array()?
            .iter()
            .filter_map(|card| {
                let id = card.get("id")?.as_str()?.to_string();
                let name = card
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&id)
                    .to_string();
                Some((id, name))
            })
            .collect();

        Some(RewardSnapshot {
            choices,
            deck_counts: deck_counts(state),
        })
    }
}

fn infer_reward_choice(
    snapshot: &RewardSnapshot,
    state: &Value,
    pm: &crate::locales::PostmortemLocale,
) -> Option<String> {
    let next_counts = deck_counts(state);

    for (id, name) in &snapshot.choices {
        let before = snapshot.deck_counts.get(id).copied().unwrap_or(0);
        let after = next_counts.get(id).copied().unwrap_or(0);
        if after > before {
            return Some(pm.label_picked.replace("{name}", name));
        }
    }

    if next_counts == snapshot.deck_counts {
        return Some(pm.label_skipped.clone());
    }

    None
}

fn deck_counts(state: &Value) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    if let Some(cards) = state.get("master_cards").and_then(|v| v.as_array()) {
        for card in cards {
            if let Some(id) = card.get("id").and_then(|v| v.as_str()) {
                *counts.entry(id.to_string()).or_insert(0) += 1;
            }
        }
    }
    counts
}

fn display_i64(state: &Value, key: &str) -> String {
    state
        .get(key)
        .and_then(|v| v.as_i64())
        .map_or("?".to_string(), |v| v.to_string())
}

#[cfg(test)]
#[path = "tests/postmortem_tests.rs"]
mod tests;
