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

pub fn combine_postmortem_report(
    ai_report: &str,
    deterministic_report: &str,
    section_machine: &str,
) -> String {
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

fn tmp_path(path: &Path) -> std::path::PathBuf {
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

#[derive(Debug, Default)]
struct CombatRecord {
    room_type: String,
    start_hp: Option<i64>,
    end_hp: Option<i64>,
    monsters: Vec<String>,
    is_fatal: bool,
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
    let mut run_ended_reason = None;
    let mut run_metadata = None;
    let mut final_state = None;
    let mut advice_lines = Vec::new();
    let mut reward_lines = Vec::new();
    let mut pending_reward: Option<RewardSnapshot> = None;
    let mut global_monster_names: Vec<String> = Vec::new();
    let mut combat_records: Vec<CombatRecord> = Vec::new();
    let mut in_combat = false;
    let mut current_combat: CombatRecord = CombatRecord::default();
    let mut seen_indexes: Vec<usize> = Vec::new();
    let mut elite_count = 0usize;
    let mut boss_count = 0usize;
    let mut normal_count = 0usize;

    let pm = &locale.postmortem;

    for event in &events {
        match event.get("event").and_then(|v| v.as_str()) {
            Some("run_started") => run_started = event.get("ts_ms").and_then(|v| v.as_i64()),
            Some("run_ended") => {
                run_ended_reason = event
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
                        if !in_combat {
                            in_combat = true;
                            let room_type = state
                                .get("room_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?")
                                .to_string();
                            let mut combat_monsters = Vec::new();
                            if let Some(monsters) = state.get("monsters").and_then(|v| v.as_array())
                            {
                                for m in monsters {
                                    if let Some(name) = m.get("name").and_then(|v| v.as_str()) {
                                        combat_monsters.push(name.to_string());
                                    }
                                    if let Some(idx) = m.get("index").and_then(|v| v.as_u64()) {
                                        let idx = idx as usize;
                                        if !seen_indexes.contains(&idx) {
                                            seen_indexes.push(idx);
                                        }
                                    }
                                }
                            }
                            current_combat = CombatRecord {
                                room_type,
                                start_hp: state.get("current_hp").and_then(|v| v.as_i64()),
                                end_hp: None,
                                monsters: combat_monsters,
                                is_fatal: false,
                            };
                        } else if let Some(monsters) =
                            state.get("monsters").and_then(|v| v.as_array())
                        {
                            for m in monsters {
                                if let Some(idx) = m.get("index").and_then(|v| v.as_u64())
                                    && !seen_indexes.contains(&(idx as usize))
                                {
                                    seen_indexes.push(idx as usize);
                                    if let Some(name) = m.get("name").and_then(|v| v.as_str())
                                        && !current_combat.monsters.contains(&name.to_string())
                                    {
                                        current_combat.monsters.push(name.to_string());
                                    }
                                }
                            }
                        }

                        if let Some(monsters) = state.get("monsters").and_then(|v| v.as_array()) {
                            for m in monsters {
                                if let Some(name) = m.get("name").and_then(|v| v.as_str())
                                    && !global_monster_names.contains(&name.to_string())
                                {
                                    global_monster_names.push(name.to_string());
                                }
                            }
                        }
                    }

                    if screen != "BATTLE"
                        && screen != "NONE"
                        && screen != "HAND_SELECT"
                        && in_combat
                    {
                        current_combat.end_hp = state.get("current_hp").and_then(|v| v.as_i64());
                        if current_combat.end_hp == Some(0) {
                            current_combat.is_fatal = true;
                        }
                        match current_combat.room_type.as_str() {
                            "MonsterRoomElite" | "MonsterRoomBoss" => {
                                if current_combat.room_type.contains("Boss") {
                                    boss_count += 1;
                                } else {
                                    elite_count += 1;
                                }
                            }
                            _ => normal_count += 1,
                        }
                        combat_records.push(std::mem::take(&mut current_combat));
                        in_combat = false;
                        seen_indexes.clear();
                    }

                    final_state = Some(state.clone());
                }
            }
            _ => {}
        }
    }

    if in_combat {
        combat_records.push(current_combat);
    }

    let mut report = Vec::new();
    report.push(pm.report_title.clone());
    report.push(String::new());
    report.push(pm.run_section.clone());
    if let Some(ts) = run_started {
        report.push(pm.label_started.replace("{ts}", &ts.to_string()));
    }

    let death_cause = final_state.as_ref().and_then(|state| {
        let hp = state.get("current_hp").and_then(|v| v.as_i64());
        if hp == Some(0) {
            let floor = state
                .get("floor")
                .and_then(|v| v.as_i64())
                .map_or("?".to_string(), |v| v.to_string());
            let room = state
                .get("room_type")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let monsters: Vec<String> = state
                .get("monsters")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            m.get("name")
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let monster_list = if monsters.is_empty() {
                "?".to_string()
            } else {
                let mut counts: HashMap<&str, usize> = HashMap::new();
                for m in &monsters {
                    *counts.entry(m.as_str()).or_insert(0) += 1;
                }
                let mut entries: Vec<String> = counts
                    .iter()
                    .map(|(name, &count)| {
                        if count > 1 {
                            format!("{name}×{count}")
                        } else {
                            name.to_string()
                        }
                    })
                    .collect();
                entries.sort();
                entries.join("、")
            };
            Some(format!("第{floor}层 {room} — 死于 {monster_list}"))
        } else {
            None
        }
    });

    if let Some(ref cause) = death_cause {
        report.push(pm.label_death.replace("{cause}", cause));
    }

    let is_victory = final_state.as_ref().is_some_and(|s| {
        s.get("current_hp").and_then(|v| v.as_i64()).unwrap_or(0) > 0
            && run_ended_reason.as_deref() == Some("game_over")
    });

    if is_victory {
        let room = final_state
            .as_ref()
            .and_then(|s| s.get("room_type"))
            .and_then(|v| v.as_str());
        if room == Some("VictoryRoom") {
            report.push(format!("{} — Heart defeated!", pm.label_victory));
        } else {
            report.push(pm.label_victory.clone());
        }
    } else if let Some(reason) = run_ended_reason.as_ref() {
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

    if !combat_records.is_empty() {
        report.push(String::new());
        report.push(pm.section_monsters.clone());
        let total = combat_records.len();
        report.push(
            pm.label_combats_summary
                .replace("{n}", &total.to_string())
                .replace("{list}", &global_monster_names.join(", ")),
        );
        report.push(
            pm.label_combat_type_count
                .replace("{normal}", &normal_count.to_string())
                .replace("{elite}", &elite_count.to_string())
                .replace("{boss}", &boss_count.to_string()),
        );

        for (i, combat) in combat_records.iter().enumerate() {
            if let (Some(start), Some(end)) = (combat.start_hp, combat.end_hp) {
                let delta = end - start;
                let hp_change = if delta >= 0 {
                    format!("+{delta}")
                } else {
                    delta.to_string()
                };
                let monster_list = if combat.monsters.is_empty() {
                    String::new()
                } else {
                    let mut name_counts: HashMap<&str, usize> = HashMap::new();
                    for m in &combat.monsters {
                        *name_counts.entry(m.as_str()).or_insert(0) += 1;
                    }
                    let mut entries: Vec<String> = name_counts
                        .into_iter()
                        .map(|(name, count)| {
                            if count > 1 {
                                format!("{name}×{count}")
                            } else {
                                name.to_string()
                            }
                        })
                        .collect();
                    entries.sort();
                    entries.join("、")
                };

                let label = match combat.room_type.as_str() {
                    "MonsterRoomElite" => &pm.label_combat_elite,
                    "MonsterRoomBoss" => &pm.label_combat_boss,
                    _ => &pm.label_combat_hp,
                };
                report.push(
                    (if combat.is_fatal {
                        format!("💀 {label}")
                    } else {
                        label.clone()
                    })
                    .replace("{n}", &(i + 1).to_string())
                    .replace("{start}", &start.to_string())
                    .replace("{end}", &end.to_string())
                    .replace("{delta}", &hp_change)
                    .replace("{monsters}", &monster_list),
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
