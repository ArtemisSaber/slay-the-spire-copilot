use super::{RewardSnapshot, infer_reward_choice};
use crate::locales::PostmortemLocale;
use serde_json::Value;

#[derive(Debug, Default)]
pub(crate) struct CombatRecord {
    pub(crate) room_type: String,
    pub(crate) start_hp: Option<i64>,
    pub(crate) end_hp: Option<i64>,
    pub(crate) monsters: Vec<String>,
    pub(crate) is_fatal: bool,
}

#[derive(Debug, Default)]
pub(crate) struct JournalSummary {
    pub(crate) malformed: usize,
    pub(crate) run_started: Option<i64>,
    pub(crate) run_ended_reason: Option<String>,
    pub(crate) run_metadata: Option<Value>,
    pub(crate) final_state: Option<Value>,
    pub(crate) advice_lines: Vec<String>,
    pub(crate) reward_lines: Vec<String>,
    pub(crate) global_monster_names: Vec<String>,
    pub(crate) combat_records: Vec<CombatRecord>,
    pub(crate) elite_count: usize,
    pub(crate) boss_count: usize,
    pub(crate) normal_count: usize,
}

pub(crate) fn collect_journal(
    input: &str,
    pm: &PostmortemLocale,
) -> Result<JournalSummary, String> {
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

    let mut summary = JournalSummary {
        malformed,
        ..Default::default()
    };
    let mut pending_reward: Option<RewardSnapshot> = None;
    let mut in_combat = false;
    let mut current_combat = CombatRecord::default();
    let mut seen_indexes = Vec::new();

    for event in &events {
        match event.get("event").and_then(|v| v.as_str()) {
            Some("run_started") => {
                summary.run_started = event.get("ts_ms").and_then(|v| v.as_i64())
            }
            Some("run_ended") => {
                summary.run_ended_reason = event
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            }
            Some("run_metadata") => {
                summary.run_metadata = Some(event.clone());
            }
            Some("advice") => {
                let hash = event
                    .get("advice_hash")
                    .or_else(|| event.get("state_hash"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let advice = event.get("advice").and_then(|v| v.as_str()).unwrap_or("");
                summary
                    .advice_lines
                    .push(format!("- `{hash}`:\n  {}", advice.replace('\n', "\n  ")));
            }
            Some("state_changed") => {
                if let Some(state) = event.get("normalized") {
                    if let Some(snapshot) = pending_reward.take()
                        && let Some(line) = infer_reward_choice(&snapshot, state, pm)
                    {
                        summary.reward_lines.push(line);
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
                        collect_combat_state(
                            state,
                            &mut in_combat,
                            &mut current_combat,
                            &mut seen_indexes,
                            &mut summary.global_monster_names,
                        );
                    }

                    if screen != "BATTLE"
                        && screen != "NONE"
                        && screen != "HAND_SELECT"
                        && in_combat
                    {
                        finish_combat(
                            state,
                            &mut current_combat,
                            &mut summary.combat_records,
                            &mut summary.normal_count,
                            &mut summary.elite_count,
                            &mut summary.boss_count,
                        );
                        in_combat = false;
                        seen_indexes.clear();
                    }

                    summary.final_state = Some(state.clone());
                }
            }
            _ => {}
        }
    }

    if in_combat {
        summary.combat_records.push(current_combat);
    }

    Ok(summary)
}

fn collect_combat_state(
    state: &Value,
    in_combat: &mut bool,
    current_combat: &mut CombatRecord,
    seen_indexes: &mut Vec<usize>,
    global_monster_names: &mut Vec<String>,
) {
    if !*in_combat {
        *in_combat = true;
        let room_type = state
            .get("room_type")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let mut combat_monsters = Vec::new();
        if let Some(monsters) = state.get("monsters").and_then(|v| v.as_array()) {
            for monster in monsters {
                if let Some(name) = monster.get("name").and_then(|v| v.as_str()) {
                    combat_monsters.push(name.to_string());
                }
                if let Some(index) = monster.get("index").and_then(|v| v.as_u64()) {
                    let index = index as usize;
                    if !seen_indexes.contains(&index) {
                        seen_indexes.push(index);
                    }
                }
            }
        }
        *current_combat = CombatRecord {
            room_type,
            start_hp: state.get("current_hp").and_then(|v| v.as_i64()),
            end_hp: None,
            monsters: combat_monsters,
            is_fatal: false,
        };
    } else if let Some(monsters) = state.get("monsters").and_then(|v| v.as_array()) {
        for monster in monsters {
            if let Some(index) = monster.get("index").and_then(|v| v.as_u64())
                && !seen_indexes.contains(&(index as usize))
            {
                seen_indexes.push(index as usize);
                if let Some(name) = monster.get("name").and_then(|v| v.as_str())
                    && !current_combat.monsters.contains(&name.to_string())
                {
                    current_combat.monsters.push(name.to_string());
                }
            }
        }
    }

    if let Some(monsters) = state.get("monsters").and_then(|v| v.as_array()) {
        for monster in monsters {
            if let Some(name) = monster.get("name").and_then(|v| v.as_str())
                && !global_monster_names.contains(&name.to_string())
            {
                global_monster_names.push(name.to_string());
            }
        }
    }
}

fn finish_combat(
    state: &Value,
    current_combat: &mut CombatRecord,
    combat_records: &mut Vec<CombatRecord>,
    normal_count: &mut usize,
    elite_count: &mut usize,
    boss_count: &mut usize,
) {
    current_combat.end_hp = state.get("current_hp").and_then(|v| v.as_i64());
    if current_combat.end_hp == Some(0) {
        current_combat.is_fatal = true;
    }
    match current_combat.room_type.as_str() {
        "MonsterRoomElite" | "MonsterRoomBoss" => {
            if current_combat.room_type.contains("Boss") {
                *boss_count += 1;
            } else {
                *elite_count += 1;
            }
        }
        _ => *normal_count += 1,
    }
    combat_records.push(std::mem::take(current_combat));
}
