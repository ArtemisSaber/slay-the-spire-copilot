use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const POSTMORTEM_FILE_NAME: &str = "postmortem.md";

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

pub fn write_report_for_journal(journal_path: &Path, report: &str) -> Result<PathBuf, String> {
    let report_path = postmortem_path_for_journal(journal_path);
    fs::write(&report_path, report).map_err(|e| e.to_string())?;
    Ok(report_path)
}

pub fn generate_report_from_jsonl(
    input: &str,
    locale: &crate::locales::Locale,
) -> Result<String, String> {
    let mut events = Vec::new();
    let mut malformed = 0usize;

    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        match serde_json::from_str::<Value>(line) {
            Ok(value) => events.push(value),
            Err(_) => malformed += 1,
        }
    }

    if events.is_empty() {
        return Err(if malformed > 0 {
            "No valid journal events found".to_string()
        } else {
            "No journal events found".to_string()
        });
    }

    let mut run_started = None;
    let mut run_ended = None;
    let mut run_metadata = None;
    let mut final_state = None;
    let mut advice_lines = Vec::new();
    let mut reward_lines = Vec::new();
    let mut pending_reward: Option<RewardSnapshot> = None;
    let mut monster_names: Vec<String> = Vec::new();
    let mut combat_count = 0usize;
    let mut in_combat = false;
    let mut combat_hp_start: Option<i64> = None;
    let mut combat_hp_history: Vec<(Option<i64>, Option<i64>)> = Vec::new();

    let pm = &locale.postmortem;

    for event in &events {
        match event.get("event").and_then(|v| v.as_str()) {
            Some("run_started") => run_started = event.get("ts_ms").and_then(|v| v.as_i64()),
            Some("run_ended") => {
                run_ended = event
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }
            Some("run_metadata") => {
                run_metadata = Some(event.clone());
            }
            Some("advice") => {
                let hash = event
                    .get("advice_hash")
                    .or_else(|| event.get("state_hash"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let advice = event.get("advice").and_then(|v| v.as_str()).unwrap_or("");
                advice_lines.push(format!("- `{hash}`:\n  {}", advice.replace('\n', "\n  ")));
            }
            Some("state_changed") => {
                if let Some(state) = event.get("normalized") {
                    if let Some(snapshot) = pending_reward.take()
                        && let Some(line) = infer_reward_choice(&snapshot, state, pm)
                    {
                        reward_lines.push(line);
                    }

                    if state.get("screen_type").and_then(|v| v.as_str()) == Some("CARD_REWARD") {
                        pending_reward = RewardSnapshot::from_state(state);
                    }

                    let screen = state
                        .get("screen_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("NONE");

                    let has_monsters = state
                        .get("monsters")
                        .and_then(|v| v.as_array())
                        .is_some_and(|arr| !arr.is_empty());

                    if has_monsters {
                        if let Some(monsters) = state.get("monsters").and_then(|v| v.as_array()) {
                            for m in monsters {
                                if let Some(name) = m.get("name").and_then(|v| v.as_str())
                                    && !monster_names.contains(&name.to_string())
                                {
                                    monster_names.push(name.to_string());
                                }
                            }
                        }

                        if !in_combat {
                            in_combat = true;
                            combat_count += 1;
                            combat_hp_start = state.get("current_hp").and_then(|v| v.as_i64());
                        }
                    }

                    if screen != "BATTLE" && screen != "NONE" && in_combat {
                        let final_hp = state.get("current_hp").and_then(|v| v.as_i64());
                        combat_hp_history.push((combat_hp_start, final_hp));
                        in_combat = false;
                        combat_hp_start = None;
                    }

                    final_state = Some(state.clone());
                }
            }
            _ => {}
        }
    }

    let mut report = Vec::new();
    report.push(pm.report_title.clone());
    report.push(String::new());
    report.push(pm.run_section.clone());
    if let Some(ts) = run_started {
        report.push(pm.label_started.replace("{ts}", &ts.to_string()));
    }
    if let Some(reason) = run_ended.as_ref() {
        report.push(pm.label_ended.replace("{reason}", reason));
    }
    if malformed > 0 {
        report.push(pm.label_malformed.replace("{n}", &malformed.to_string()));
    }

    let character = run_metadata
        .as_ref()
        .and_then(|m| m.get("character").and_then(|v| v.as_str()))
        .or_else(|| {
            final_state
                .as_ref()
                .and_then(|s| s.get("character").and_then(|v| v.as_str()))
        });
    let ascension = run_metadata
        .as_ref()
        .and_then(|m| m.get("ascension_level").and_then(|v| v.as_i64()))
        .or_else(|| {
            final_state
                .as_ref()
                .and_then(|s| s.get("ascension_level").and_then(|v| v.as_i64()))
        });

    if let Some(state) = final_state.as_ref() {
        report.push(String::new());
        report.push(pm.section_overview.clone());
        if let Some(character) = character {
            report.push(pm.label_character.replace("{char}", character));
        }
        if let Some(asc) = ascension {
            report.push(pm.label_ascension.replace("{level}", &asc.to_string()));
        }
        report.push(
            pm.label_floor.replace(
                "{floor}",
                &state
                    .get("floor")
                    .and_then(|v| v.as_i64())
                    .map_or("?".to_string(), |v| v.to_string()),
            ),
        );
        report.push(
            pm.label_hp
                .replace("{cur}", &display_i64(state, "current_hp"))
                .replace("{max}", &display_i64(state, "max_hp")),
        );
        report.push(pm.label_gold.replace("{gold}", &display_i64(state, "gold")));
        report.push(
            pm.label_deck_count.replace(
                "{n}",
                &state
                    .get("master_cards")
                    .and_then(|v| v.as_array())
                    .map_or(0, |arr| arr.len())
                    .to_string(),
            ),
        );
        report.push(
            pm.label_relics_count.replace(
                "{n}",
                &state
                    .get("relics")
                    .and_then(|v| v.as_array())
                    .map_or(0, |arr| arr.len())
                    .to_string(),
            ),
        );
    } else {
        report.push(String::new());
        report.push(pm.section_overview.clone());
        if let Some(character) = character {
            report.push(pm.label_character.replace("{char}", character));
        }
        if let Some(asc) = ascension {
            report.push(pm.label_ascension.replace("{level}", &asc.to_string()));
        }
    }

    if let Some(state) = final_state.as_ref() {
        if let Some(relics) = state.get("relics").and_then(|v| v.as_array())
            && !relics.is_empty()
        {
            report.push(String::new());
            report.push(pm.section_relics.clone());
            for relic in relics {
                if let Some(name) = relic.get("name").and_then(|v| v.as_str()) {
                    let desc = relic
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    report.push(format!("- {name}: {desc}"));
                }
            }
        }

        if let Some(potions) = state.get("potions").and_then(|v| v.as_array())
            && !potions.is_empty()
        {
            report.push(String::new());
            report.push(pm.section_potions.clone());
            for potion in potions {
                if let Some(name) = potion.get("name").and_then(|v| v.as_str()) {
                    let desc = potion
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    report.push(format!("- {name}: {desc}"));
                }
            }
        }

        if let Some(deck_names) = state.get("deck_names").and_then(|v| v.as_array())
            && !deck_names.is_empty()
        {
            let mut cards: Vec<String> = deck_names
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            cards.sort();
            cards.dedup();
            let mut counts = std::collections::HashMap::new();
            for card in deck_names.iter().filter_map(|v| v.as_str()) {
                *counts.entry(card.to_string()).or_insert(0) += 1;
            }
            report.push(String::new());
            report.push(pm.section_deck.clone());
            for name in &cards {
                let count = counts.get(name).copied().unwrap_or(1);
                if count > 1 {
                    report.push(format!("- {name} ×{count}"));
                } else {
                    report.push(format!("- {name}"));
                }
            }
        }
    }

    if !monster_names.is_empty() {
        report.push(String::new());
        report.push(pm.section_monsters.clone());
        report.push(
            pm.label_combats_summary
                .replace("{n}", &combat_count.to_string())
                .replace("{list}", &monster_names.join(", ")),
        );
        for (i, (start_hp, end_hp)) in combat_hp_history.iter().enumerate() {
            if let (Some(start), Some(end)) = (start_hp, end_hp) {
                let delta = end - start;
                let hp_change = if delta >= 0 {
                    format!("+{delta}")
                } else {
                    delta.to_string()
                };
                report.push(
                    pm.label_combat_hp
                        .replace("{n}", &(i + 1).to_string())
                        .replace("{start}", &start.to_string())
                        .replace("{end}", &end.to_string())
                        .replace("{delta}", &hp_change),
                );
            }
        }
    }

    if !advice_lines.is_empty() {
        report.push(String::new());
        report.push(pm.section_decisions.clone());
        report.extend(advice_lines);
    }

    if !reward_lines.is_empty() {
        report.push(String::new());
        report.push(pm.section_rewards.clone());
        report.extend(reward_lines);
    }

    Ok(report.join("\n"))
}

pub fn build_ai_postmortem_prompt(
    deterministic_report: &str,
    locale: &crate::locales::Locale,
) -> String {
    let pm = &locale.postmortem;
    let lang_name = &locale.language_name;
    format!(
        "{}\n\n{}\n{}\n{}\n{}\n{}\n\n{}\n{deterministic_report}\n",
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
