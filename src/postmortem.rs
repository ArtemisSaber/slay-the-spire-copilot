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

#[derive(Debug, Default)]
struct CombatRecord {
    room_type: String,
    start_hp: Option<i64>,
    end_hp: Option<i64>,
    monsters: Vec<String>,
    is_fatal: bool,
}

struct NormalizedSlice<'a> {
    data: Option<&'a Value>,
    hash: &'a str,
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

    let mut route_lines: Vec<String> = Vec::new();
    let mut rest_lines: Vec<String> = Vec::new();
    let mut shop_lines: Vec<String> = Vec::new();
    let mut potion_lines: Vec<String> = Vec::new();
    let mut boss_relic_lines: Vec<String> = Vec::new();

    let mut prev_floor: Option<i64> = None;
    let mut prev_hp: Option<i64> = None;
    let mut prev_gold: Option<i64> = None;
    let mut prev_relics: Vec<String> = Vec::new();
    let mut prev_potions: Vec<String> = Vec::new();
    let mut prev_deck: HashMap<String, usize> = HashMap::new();
    let mut pending_rest: Option<i64> = None;
    let mut pending_shop: Option<i64> = None;
    let mut pending_boss_relic: Option<Vec<(String, String)>> = None;

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

                    let floor = state.get("floor").and_then(|v| v.as_i64());
                    let hp = state.get("current_hp").and_then(|v| v.as_i64());
                    let gold = state.get("gold").and_then(|v| v.as_i64());

                    let current_relics: Vec<String> = state
                        .get("relics")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|r| {
                                    r.get("name")
                                        .and_then(|n| n.as_str())
                                        .map(|s| s.to_string())
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    let current_potions: Vec<String> = state
                        .get("potions")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|p| {
                                    p.get("name")
                                        .and_then(|n| n.as_str())
                                        .map(|s| s.to_string())
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    let current_deck = deck_counts(state);

                    if screen == "MAP"
                        && state.get("map_first_node_chosen").and_then(|v| v.as_bool())
                            == Some(true)
                        && let (Some(cx), Some(cy)) = (
                            state.get("map_current_x").and_then(|v| v.as_i64()),
                            state.get("map_current_y").and_then(|v| v.as_i64()),
                        )
                    {
                        let room = state
                            .get("room_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        let symbol = state
                            .get("map_nodes")
                            .and_then(|v| v.as_array())
                            .and_then(|arr| {
                                arr.iter().find_map(|n| {
                                    if n.get("x").and_then(|v| v.as_i64()) == Some(cx)
                                        && n.get("y").and_then(|v| v.as_i64()) == Some(cy)
                                    {
                                        n.get("symbol")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string())
                                    } else {
                                        None
                                    }
                                })
                            })
                            .unwrap_or_else(|| room.to_string());
                        if prev_floor != floor {
                            route_lines.push(format_route_entry(&symbol, room));
                            prev_floor = floor;
                        }
                    }

                    if floor == Some(0) && route_lines.is_empty() {
                        route_lines.push("起始".to_string());
                        prev_floor = Some(0);
                    }

                    if screen != "REST" && screen != "SHOP" && screen != "BOSS_REWARD" {
                        if let Some(rest_floor) = pending_rest.take()
                            && let (Some(prev_hp), Some(hp)) = (prev_hp, hp)
                        {
                            let healed = hp.saturating_sub(prev_hp);
                            let is_shop_or_event = screen == "SHOP" || screen == "EVENT";
                            if healed > 0 && !is_shop_or_event {
                                rest_lines.push(
                                    pm.label_rest_entry
                                        .replace("{floor}", &rest_floor.to_string())
                                        .replace("{choice}", &pm.label_rest_rested.clone()),
                                );
                            } else {
                                rest_lines.push(
                                    pm.label_rest_entry
                                        .replace("{floor}", &rest_floor.to_string())
                                        .replace("{choice}", &pm.label_rest_smithed.clone()),
                                );
                            }
                        }

                        if let Some(shop_floor) = pending_shop.take() {
                            let mut bought = Vec::new();
                            for (id, count) in &current_deck {
                                let prev = prev_deck.get(id).copied().unwrap_or(0);
                                if *count > prev
                                    && let Some(name) = card_name_from_master_cards(state, id)
                                {
                                    bought.push(name);
                                }
                            }
                            let mut removed = Vec::new();
                            for (id, count) in &prev_deck {
                                let curr = current_deck.get(id).copied().unwrap_or(0);
                                if curr < *count {
                                    removed.push(id.clone());
                                }
                            }
                            for relic in &current_relics {
                                if !prev_relics.contains(relic) {
                                    bought.push(format!("[遗物] {relic}"));
                                }
                            }
                            let spent = prev_gold
                                .and_then(|pg| gold.map(|g| pg.saturating_sub(g)))
                                .unwrap_or(0);
                            if spent > 0 || !bought.is_empty() || !removed.is_empty() {
                                shop_lines.push(
                                    pm.label_shop_spent
                                        .replace("{floor}", &shop_floor.to_string())
                                        .replace("{gold}", &spent.to_string()),
                                );
                                for item in &bought {
                                    shop_lines.push(pm.label_shop_bought.replace("{item}", item));
                                }
                                for card_id in &removed {
                                    let name = card_name_from_deck_map(&prev_deck, card_id)
                                        .unwrap_or_else(|| card_id.clone());
                                    shop_lines.push(pm.label_shop_removed.replace("{card}", &name));
                                }
                            }
                        }

                        if let Some(choices) = pending_boss_relic.take() {
                            for relic in &current_relics {
                                if !prev_relics.contains(relic) {
                                    for (_, name) in &choices {
                                        if relic == name {
                                            boss_relic_lines.push(
                                                pm.label_boss_relic_entry.replace("{name}", name),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if screen == "REST" {
                        pending_rest = floor;
                    }
                    if screen == "SHOP" {
                        pending_shop = floor;
                    }
                    if screen == "BOSS_REWARD" {
                        let choices = extract_boss_relic_choices(state);
                        if !choices.is_empty() {
                            pending_boss_relic = Some(choices);
                        }
                    }

                    if !prev_potions.is_empty() {
                        for potion in &prev_potions {
                            if !current_potions.contains(potion) {
                                let floor_label = floor.map_or("?".to_string(), |f| f.to_string());
                                potion_lines.push(
                                    pm.label_potion_used
                                        .replace("{name}", potion)
                                        .replace("{floor}", &floor_label),
                                );
                            }
                        }
                    }

                    prev_hp = hp;
                    prev_gold = gold;
                    prev_relics = current_relics;
                    prev_potions = current_potions;
                    prev_deck = current_deck;

                    final_state = Some(state.clone());
                }
            }
            _ => {}
        }
    }

    if in_combat {
        combat_records.push(current_combat);
    }

    if let Some(ref state) = final_state
        && let (Some(final_floor), Some(last_route_floor)) =
            (state.get("floor").and_then(|v| v.as_i64()), prev_floor)
        && final_floor > last_route_floor
    {
        let room = state
            .get("room_type")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        route_lines.push(room_display(room).to_string());
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
        if let Some(reason) = run_ended_reason.as_ref() {
            report.push(pm.label_ended.replace("{reason}", reason));
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
        report.extend(advice_lines.clone());
    }

    if !reward_lines.is_empty() {
        report.push(String::new());
        report.push(pm.section_rewards.clone());
        report.extend(reward_lines);
    }

    if !route_lines.is_empty() {
        report.push(String::new());
        report.push(pm.section_route.clone());
        report.push(route_lines.join(" → "));
    }

    if !rest_lines.is_empty() {
        report.push(String::new());
        report.push(pm.section_rest_choices.clone());
        report.extend(rest_lines);
    }

    if !shop_lines.is_empty() {
        report.push(String::new());
        report.push(pm.section_shop.clone());
        report.extend(shop_lines);
    }

    if !boss_relic_lines.is_empty() {
        report.push(String::new());
        report.push(pm.section_boss_relic.clone());
        report.extend(boss_relic_lines);
    }

    let final_combat_lines = build_final_combat_lines(&events, &advice_lines, pm);
    if !final_combat_lines.is_empty() {
        report.push(String::new());
        report.extend(final_combat_lines);
    }

    if !potion_lines.is_empty() {
        report.push(String::new());
        report.push(pm.section_potions.clone());
        report.extend(potion_lines);
    }

    Ok(report.join("\n"))
}

fn build_final_combat_lines(
    events: &[Value],
    advice_lines: &[String],
    pm: &crate::locales::PostmortemLocale,
) -> Vec<String> {
    let fatal_floor = events
        .iter()
        .filter_map(|e| e.get("normalized"))
        .find(|n| n.get("current_hp").and_then(|v| v.as_i64()) == Some(0))
        .and_then(|n| n.get("floor").and_then(|v| v.as_i64()));

    let Some(fatal_floor) = fatal_floor else {
        return Vec::new();
    };

    let states: Vec<&Value> = events
        .iter()
        .filter(|e| {
            e.get("normalized")
                .and_then(|n| n.get("floor").and_then(|v| v.as_i64()))
                == Some(fatal_floor)
        })
        .collect();

    if states.is_empty() {
        return Vec::new();
    }

    let monsters: Vec<String> = states
        .first()
        .and_then(|e| e.get("normalized"))
        .and_then(|s| s.get("monsters").and_then(|v| v.as_array()))
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    m.get("name")
                        .and_then(|n| n.as_str().map(|s| s.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();

    let monster_names = monsters.join("、");

    let mut lines = Vec::new();
    lines.push(format!("{} — {monster_names}", pm.section_final_combat));

    let mut turn_states: Vec<Vec<NormalizedSlice>> = Vec::new();
    let mut current_turn: Vec<NormalizedSlice> = Vec::new();
    let mut last_turn: Option<i64> = None;

    for event in &states {
        let n = event.get("normalized");
        let turn = n.and_then(|n| n.get("turn").and_then(|v| v.as_i64()));
        let energy = n.and_then(|n| n.get("energy").and_then(|v| v.as_i64()));
        let hash = event
            .get("advice_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let slice = NormalizedSlice { data: n, hash };

        let is_new_turn = if turn.is_some() {
            turn != last_turn && !current_turn.is_empty()
        } else {
            // Fallback: energy reset or hand significantly larger
            let prev_energy = current_turn
                .last()
                .and_then(|s| s.data)
                .and_then(|v| v.get("energy").and_then(|v| v.as_i64()))
                .unwrap_or(0);
            let prev_hand = current_turn
                .last()
                .and_then(|s| s.data)
                .and_then(|v| v.get("hand").and_then(|v| v.as_array()))
                .map(|a| a.len())
                .unwrap_or(0);
            let cur_hand = n
                .and_then(|v| v.get("hand").and_then(|v| v.as_array()))
                .map(|a| a.len())
                .unwrap_or(0);
            (energy.unwrap_or(0) > prev_energy + 1 || cur_hand > prev_hand + 2)
                && !current_turn.is_empty()
        };

        if is_new_turn {
            turn_states.push(std::mem::take(&mut current_turn));
        }
        current_turn.push(slice);
        last_turn = turn;
    }
    if !current_turn.is_empty() {
        turn_states.push(current_turn);
    }

    let mut prev_potions: Vec<String> = states
        .first()
        .and_then(|e| e.get("normalized"))
        .and_then(|s| s.get("potions").and_then(|v| v.as_array()))
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    p.get("name")
                        .and_then(|n| n.as_str().map(|s| s.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();

    for (ti, turn) in turn_states.iter().enumerate() {
        let first = turn.first().and_then(|s| s.data).unwrap();
        let last = turn.last().and_then(|s| s.data).unwrap();
        let state_hash = turn.first().map(|s| s.hash).unwrap_or("");

        let hp = first
            .get("current_hp")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let block = first.get("block").and_then(|v| v.as_i64()).unwrap_or(0);

        let hand: Vec<String> = first
            .get("hand")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        c.get("name")
                            .and_then(|n| n.as_str().map(|s| s.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let incoming: i64 = first
            .get("incoming_damage")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let monster_hp: Vec<String> = first
            .get("monsters")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|m| {
                        let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                        let hp = m.get("current_hp").and_then(|v| v.as_i64()).unwrap_or(0);
                        format!("{name}({hp})")
                    })
                    .collect()
            })
            .unwrap_or_default();

        let intent_str = if incoming > 0 {
            "攻击".to_string()
        } else {
            "减益".to_string()
        };

        lines.push(String::new());
        lines.push(
            pm.label_final_turn
                .replace("{n}", &(ti + 1).to_string())
                .replace("{hp}", &hp.to_string())
                .replace("{block}", &block.to_string())
                .replace("{hand}", &hand.join(", ")),
        );

        if !monster_hp.is_empty() || incoming != 0 {
            let mon_hp_str = monster_hp.join(", ");
            lines.push(
                pm.label_final_enemy
                    .replace("{intent}", &intent_str)
                    .replace("{dmg}", &incoming.to_string())
                    .replace("{hp}", &mon_hp_str),
            );
        }

        if !state_hash.is_empty() {
            for adv in advice_lines {
                if adv.contains(state_hash) {
                    let short = adv
                        .lines()
                        .skip(1)
                        .find(|l| !l.trim().is_empty())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if !short.is_empty() {
                        lines.push(pm.label_final_ai.replace("{advice}", &short));
                    }
                    break;
                }
            }
        }

        if turn.len() > 1 {
            let mut played = Vec::new();
            for w in turn.windows(2) {
                let prev = w[0].data;
                let curr = w[1].data;
                let prev_energy = prev
                    .and_then(|v| v.get("energy").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                let curr_energy = curr
                    .and_then(|v| v.get("energy").and_then(|v| v.as_i64()))
                    .unwrap_or(0);
                let prev_hand: Vec<(String, String)> = prev
                    .and_then(|v| v.get("hand"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .map(|c| {
                                let name = c
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("?")
                                    .to_string();
                                let uuid = c
                                    .get("uuid")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("?")
                                    .to_string();
                                (name, uuid)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let curr_hand: Vec<(String, String)> = curr
                    .and_then(|v| v.get("hand"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .map(|c| {
                                let name = c
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("?")
                                    .to_string();
                                let uuid = c
                                    .get("uuid")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("?")
                                    .to_string();
                                (name, uuid)
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let curr_uuids: std::collections::HashSet<&str> =
                    curr_hand.iter().map(|(_, u)| u.as_str()).collect();
                for (name, uuid) in &prev_hand {
                    if !curr_uuids.contains(uuid.as_str()) && curr_energy < prev_energy {
                        played.push(name.clone());
                    }
                }
            }
            if !played.is_empty() {
                lines.push(
                    pm.label_final_played
                        .replace("{cards}", &played.join(" → ")),
                );
            }
        }

        let end_hp = last.get("current_hp").and_then(|v| v.as_i64()).unwrap_or(0);
        let end_block = last.get("block").and_then(|v| v.as_i64()).unwrap_or(0);

        if end_hp == 0 {
            lines.push("  💀 HP 0, 阵亡".to_string());
        } else {
            lines.push(
                pm.label_final_result
                    .replace("{hp}", &end_hp.to_string())
                    .replace("{block}", &end_block.to_string()),
            );
        }

        let current_potions: Vec<String> = last
            .get("potions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| {
                        p.get("name")
                            .and_then(|n| n.as_str().map(|s| s.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();

        for p in &prev_potions {
            if !current_potions.contains(p) {
                lines.push(format!("  ⚠ 使用了药水: {p}"));
            }
        }
        prev_potions = current_potions;
    }

    lines
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

fn extract_boss_relic_choices(state: &Value) -> Vec<(String, String)> {
    state
        .get("boss_relic_choices")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|r| {
                    let name = r
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())?;
                    Some((name.clone(), name))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn card_name_from_master_cards(state: &Value, id: &str) -> Option<String> {
    state
        .get("master_cards")
        .and_then(|v| v.as_array())?
        .iter()
        .find_map(|card| {
            if card.get("id").and_then(|v| v.as_str()) == Some(id) {
                card.get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
}

fn card_name_from_deck_map(deck: &HashMap<String, usize>, id: &str) -> Option<String> {
    deck.keys().find(|k| k == &id).cloned()
}

fn room_display(room_type: &str) -> &str {
    match room_type {
        "MonsterRoom" => "战斗",
        "MonsterRoomElite" => "精英",
        "MonsterRoomBoss" => "Boss",
        "EventRoom" => "事件",
        "ShopRoom" => "商店",
        "RestRoom" => "休息",
        "TreasureRoom" => "宝箱",
        "NeowRoom" => "起始",
        _ => room_type,
    }
}

fn format_route_entry(symbol: &str, room_type: &str) -> String {
    if symbol == "?" {
        format!("?({})", room_display(room_type))
    } else {
        room_display(room_type).to_string()
    }
}

#[cfg(test)]
#[path = "tests/postmortem_tests.rs"]
mod tests;
