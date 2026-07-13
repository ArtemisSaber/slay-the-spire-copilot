use super::super::aggregation::JournalSummary;
use super::super::display_i64;
use crate::locales::PostmortemLocale;
use serde_json::Value;
use std::collections::HashMap;

pub(super) fn append_run_details(
    report: &mut Vec<String>,
    summary: &JournalSummary,
    pm: &PostmortemLocale,
) {
    if let Some(ts) = summary.run_started {
        report.push(pm.label_started.replace("{ts}", &ts.to_string()));
    }

    if let Some(cause) = death_cause(summary.final_state.as_ref()) {
        report.push(pm.label_death.replace("{cause}", &cause));
    }

    let is_victory = summary.final_state.as_ref().is_some_and(|state| {
        state
            .get("current_hp")
            .and_then(|value| value.as_i64())
            .unwrap_or(0)
            > 0
            && summary.run_ended_reason.as_deref() == Some("game_over")
    });
    if is_victory {
        let room = summary
            .final_state
            .as_ref()
            .and_then(|state| state.get("room_type"))
            .and_then(|value| value.as_str());
        if room == Some("VictoryRoom") {
            report.push(format!("{} — Heart defeated!", pm.label_victory));
        } else {
            report.push(pm.label_victory.clone());
        }
    } else if let Some(reason) = summary.run_ended_reason.as_ref() {
        report.push(pm.label_ended.replace("{reason}", reason));
    }

    if summary.malformed > 0 {
        report.push(
            pm.label_malformed
                .replace("{n}", &summary.malformed.to_string()),
        );
    }
}

pub(super) fn append_overview(
    report: &mut Vec<String>,
    summary: &JournalSummary,
    pm: &PostmortemLocale,
) {
    let character = summary
        .run_metadata
        .as_ref()
        .and_then(|metadata| metadata.get("character").and_then(|value| value.as_str()))
        .or_else(|| {
            summary
                .final_state
                .as_ref()
                .and_then(|state| state.get("character").and_then(|value| value.as_str()))
        });
    let ascension = summary
        .run_metadata
        .as_ref()
        .and_then(|metadata| {
            metadata
                .get("ascension_level")
                .and_then(|value| value.as_i64())
        })
        .or_else(|| {
            summary.final_state.as_ref().and_then(|state| {
                state
                    .get("ascension_level")
                    .and_then(|value| value.as_i64())
            })
        });

    report.push(String::new());
    report.push(pm.section_overview.clone());
    if let Some(character) = character {
        report.push(pm.label_character.replace("{char}", character));
    }
    if let Some(ascension) = ascension {
        report.push(
            pm.label_ascension
                .replace("{level}", &ascension.to_string()),
        );
    }

    if let Some(state) = summary.final_state.as_ref() {
        report.push(
            pm.label_floor.replace(
                "{floor}",
                &state
                    .get("floor")
                    .and_then(|value| value.as_i64())
                    .map_or("?".to_string(), |value| value.to_string()),
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
                    .and_then(|value| value.as_array())
                    .map_or(0, |cards| cards.len())
                    .to_string(),
            ),
        );
        report.push(
            pm.label_relics_count.replace(
                "{n}",
                &state
                    .get("relics")
                    .and_then(|value| value.as_array())
                    .map_or(0, |relics| relics.len())
                    .to_string(),
            ),
        );
    }
}

fn death_cause(state: Option<&Value>) -> Option<String> {
    let state = state?;
    if state.get("current_hp").and_then(|value| value.as_i64()) != Some(0) {
        return None;
    }

    let floor = state
        .get("floor")
        .and_then(|value| value.as_i64())
        .map_or("?".to_string(), |value| value.to_string());
    let room = state
        .get("room_type")
        .and_then(|value| value.as_str())
        .unwrap_or("?");
    let monsters: Vec<String> = state
        .get("monsters")
        .and_then(|value| value.as_array())
        .map(|monsters| {
            monsters
                .iter()
                .filter_map(|monster| {
                    monster
                        .get("name")
                        .and_then(|value| value.as_str().map(str::to_string))
                })
                .collect()
        })
        .unwrap_or_default();
    let monster_list = if monsters.is_empty() {
        "?".to_string()
    } else {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for monster in &monsters {
            *counts.entry(monster.as_str()).or_insert(0) += 1;
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
}
