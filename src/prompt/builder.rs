use crate::locales::Locale;
use crate::relic_counters::rewrite_relic_description;
use crate::state::{CardInfo, DangerLevel, MapCoord, MonsterInfo, NormalizedState};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::OnceLock;

use super::routing::{
    PathEvaluation, ShopTiming, enumerate_paths, enumerate_paths_from_roots, evaluate_path, plural,
};

pub(crate) const MAP_CANDIDATE_LIMIT: usize = 5;

pub(crate) fn danger_prefix(state: &NormalizedState, locale: &Locale) -> String {
    let mut reasons: Vec<&str> = Vec::new();

    if state.danger.incoming_lethal {
        reasons.push(&locale.danger.fatal_damage);
    }
    if state.danger.hp_critical {
        reasons.push(&locale.danger.hp_critical);
    }
    if state.danger.wrath_stance && state.danger.any_monster_attacking {
        reasons.push(&locale.danger.wrath_stance);
    }
    if state.danger.no_block_against_hit {
        reasons.push(&locale.danger.no_block);
    }

    match state.danger.level {
        DangerLevel::Danger => {
            if reasons.is_empty() {
                locale.danger.danger_prefix.clone()
            } else {
                locale.danger.danger_with_reasons.replace(
                    "{reasons}",
                    &reasons.join(&locale.danger.danger_reason_separator),
                )
            }
        }
        DangerLevel::Caution => locale.danger.caution.clone(),
        DangerLevel::Safe => locale.danger.safe.clone(),
    }
}

pub(crate) fn combat_profile_line(state: &NormalizedState, locale: &Locale) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(ref c) = state.character {
        let class_name = match c.as_str() {
            "IRONCLAD" => &locale.i18n.class_ironclad,
            "THE_SILENT" => &locale.i18n.class_silent,
            "DEFECT" => &locale.i18n.class_defect,
            "WATCHER" => &locale.i18n.class_watcher,
            _ => c.as_str(),
        };
        parts.push(locale.status.character.replace("{class}", class_name));
    }
    if let Some(f) = state.floor {
        parts.push(locale.status.floor.replace("{floor}", &f.to_string()));
    }
    if let Some(max) = state.max_hp {
        parts.push(locale.status.max_hp.replace("{max}", &max.to_string()));
    }
    if let Some(g) = state.gold {
        parts.push(locale.status.gold.replace("{gold}", &g.to_string()));
    }
    if !state.relics.is_empty() {
        let names: Vec<String> = state
            .relics
            .iter()
            .map(|r| match r.counter {
                Some(c) => format!("{} ({})", r.name, c),
                None => r.name.clone(),
            })
            .collect();
        parts.push(locale.status.relics.replace("{list}", &names.join(" ")));
    }

    parts.join("  ")
}

pub(crate) fn turn_status_line(state: &NormalizedState, locale: &Locale) -> String {
    let mut parts: Vec<String> = Vec::new();

    parts.push(danger_prefix(state, locale));

    if let (Some(cur), Some(max)) = (state.current_hp, state.max_hp) {
        let pct = if max > 0 {
            (cur as f64 / max as f64 * 100.0) as i64
        } else {
            0
        };
        parts.push(
            locale
                .status
                .hp
                .replace("{cur}", &cur.to_string())
                .replace("{max}", &max.to_string())
                .replace("{pct}", &pct.to_string()),
        );
    }
    if let Some(b) = state.block {
        parts.push(locale.status.block.replace("{block}", &b.to_string()));
    }
    if let Some(e) = state.energy {
        parts.push(locale.status.energy.replace("{energy}", &e.to_string()));
    }

    if !state.powers.is_empty() {
        let powers_str: Vec<String> = state
            .powers
            .iter()
            .map(|p| format!("{}({})", p.name, p.amount))
            .collect();
        parts.push(
            locale
                .status
                .powers
                .replace("{list}", &powers_str.join(" ")),
        );
    }

    if !state.potions.is_empty() {
        let names: Vec<&str> = state.potions.iter().map(|p| p.name.as_str()).collect();
        parts.push(locale.status.potions.replace("{list}", &names.join(" ")));
    }

    if state.incoming_damage > 0 {
        let warn =
            if state.block.unwrap_or(0) > 0 && state.incoming_damage > state.block.unwrap_or(0) {
                locale.status.need_block.as_str()
            } else {
                ""
            };
        parts.push(
            locale
                .status
                .damage_total
                .replace("{dmg}", &state.incoming_damage.to_string())
                .replace("{warn}", warn),
        );
    }

    parts.join("  ")
}

pub(crate) fn status_line(state: &NormalizedState, locale: &Locale) -> String {
    let mut parts: Vec<String> = Vec::new();

    parts.push(danger_prefix(state, locale));

    if let Some(ref c) = state.character {
        let class_name = match c.as_str() {
            "IRONCLAD" => &locale.i18n.class_ironclad,
            "THE_SILENT" => &locale.i18n.class_silent,
            "DEFECT" => &locale.i18n.class_defect,
            "WATCHER" => &locale.i18n.class_watcher,
            _ => c.as_str(),
        };
        parts.push(locale.status.character.replace("{class}", class_name));
    }
    if let Some(f) = state.floor {
        parts.push(locale.status.floor.replace("{floor}", &f.to_string()));
    }
    if let (Some(cur), Some(max)) = (state.current_hp, state.max_hp) {
        let pct = if max > 0 {
            (cur as f64 / max as f64 * 100.0) as i64
        } else {
            0
        };
        parts.push(
            locale
                .status
                .hp
                .replace("{cur}", &cur.to_string())
                .replace("{max}", &max.to_string())
                .replace("{pct}", &pct.to_string()),
        );
    }
    if let Some(b) = state.block {
        parts.push(locale.status.block.replace("{block}", &b.to_string()));
    }
    if let Some(e) = state.energy {
        parts.push(locale.status.energy.replace("{energy}", &e.to_string()));
    }
    if let Some(g) = state.gold {
        parts.push(locale.status.gold.replace("{gold}", &g.to_string()));
    }

    if !state.powers.is_empty() {
        let powers_str: Vec<String> = state
            .powers
            .iter()
            .map(|p| format!("{}({})", p.name, p.amount))
            .collect();
        parts.push(
            locale
                .status
                .powers
                .replace("{list}", &powers_str.join(" ")),
        );
    }

    if !state.relics.is_empty() {
        let names: Vec<String> = state
            .relics
            .iter()
            .map(|r| match r.counter {
                Some(c) => format!("{} ({})", r.name, c),
                None => r.name.clone(),
            })
            .collect();
        parts.push(locale.status.relics.replace("{list}", &names.join(" ")));
    }
    if !state.potions.is_empty() {
        let names: Vec<&str> = state.potions.iter().map(|p| p.name.as_str()).collect();
        parts.push(locale.status.potions.replace("{list}", &names.join(" ")));
    }

    if state.incoming_damage > 0 {
        let warn =
            if state.block.unwrap_or(0) > 0 && state.incoming_damage > state.block.unwrap_or(0) {
                locale.status.need_block.as_str()
            } else {
                ""
            };
        parts.push(
            locale
                .status
                .damage_total
                .replace("{dmg}", &state.incoming_damage.to_string())
                .replace("{warn}", warn),
        );
    }

    parts.join("  ")
}

pub(crate) fn clean_description(raw: &str, locale: &Locale) -> String {
    let mut result = raw.to_string();
    result = result.replace('*', "");
    let energy: &str = &locale.monster.energy_token;
    for token in &["[R]", "[G]", "[B]", "[W]", "[E]"] {
        result = result.replace(token, energy);
    }
    for n in (2..=10).rev() {
        let pattern: String = (0..n).map(|_| energy).collect::<Vec<_>>().join(" ");
        let replacement = format!("{n} {energy}");
        result = result.replace(&pattern, &replacement);
    }
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    for punct in [".", ",", ";", "!", "?", "。", "、"] {
        result = result.replace(&format!(" {punct}"), punct);
    }
    result
}

pub(crate) fn format_card(c: &CardInfo, locale: &Locale) -> String {
    let ctype = match c.card_type.as_str() {
        "ATTACK" => &locale.i18n.type_attack,
        "SKILL" => &locale.i18n.type_skill,
        "POWER" => &locale.i18n.type_power,
        "CURSE" => &locale.i18n.type_curse,
        "STATUS" => &locale.i18n.type_status,
        _ => c.card_type.as_str(),
    };
    let mut result = locale
        .card
        .format
        .replace("{up}", if c.upgraded { "+" } else { "" })
        .replace("{name}", &c.name)
        .replace("{cost}", &c.cost.to_string())
        .replace("{type}", ctype);
    if !c.description.is_empty() {
        result.push_str(
            &locale
                .card
                .with_desc
                .replace("{desc}", &clean_description(&c.description, locale)),
        );
    }
    result
}

type PowerDescMap = HashMap<String, HashMap<String, String>>;

fn power_descs() -> &'static PowerDescMap {
    static DESCS: OnceLock<PowerDescMap> = OnceLock::new();
    DESCS.get_or_init(|| {
        serde_json::from_str(include_str!("../i18n/powers.json")).unwrap_or_default()
    })
}

fn power_desc_line(id: &str, amount: i64, locale: &Locale) -> Option<String> {
    let descs = power_descs();
    let entry = descs.get(id)?;
    let template = entry.get(locale.lang_code.as_str())?;
    let desc = template.replace("{amount}", &amount.to_string());
    Some(format!("[{}]", desc))
}

pub(crate) fn format_monster(m: &MonsterInfo, locale: &Locale) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("[{}] {}", m.index, m.name));

    if let (Some(cur), Some(max)) = (m.current_hp, m.max_hp) {
        let mut hp_line = locale
            .monster
            .hp_line
            .replace("{cur}", &cur.to_string())
            .replace("{max}", &max.to_string());
        if m.can_be_killed {
            hp_line.push_str(&locale.monster.killable);
        }
        lines.push(hp_line);
    }

    if let Some(ref intent) = m.intent {
        let intent_name = match intent.as_str() {
            "ATTACK" => &locale.i18n.intent_attack,
            "ATTACK_BUFF" => &locale.i18n.intent_attack_buff,
            "ATTACK_DEBUFF" => &locale.i18n.intent_attack_debuff,
            "ATTACK_DEFEND" => &locale.i18n.intent_attack_defend,
            "BUFF" => &locale.i18n.intent_buff,
            "DEBUFF" => &locale.i18n.intent_debuff,
            "STRONG_DEBUFF" => &locale.i18n.intent_strong_debuff,
            "DEBUG" => &locale.i18n.intent_debug,
            "DEFEND" => &locale.i18n.intent_defend,
            "DEFEND_DEBUFF" => &locale.i18n.intent_defend_debuff,
            "DEFEND_BUFF" => &locale.i18n.intent_defend_buff,
            "ESCAPE" => &locale.i18n.intent_escape,
            "MAGIC" => &locale.i18n.intent_magic,
            "NONE" => &locale.i18n.intent_none,
            "SLEEP" => &locale.i18n.intent_sleep,
            "STUN" => &locale.i18n.intent_stun,
            "UNKNOWN" => &locale.i18n.intent_unknown,
            _ => intent.as_str(),
        };
        lines.push(locale.monster.intent.replace("{intent}", intent_name));
    }

    if let Some(dmg) = m.damage {
        let mut dmg_str = locale.monster.damage.replace("{dmg}", &dmg.to_string());
        if let Some(hits) = m.hits
            && hits > 1
        {
            dmg_str.push_str(
                &locale
                    .monster
                    .multi_hit
                    .replace("{hits}", &hits.to_string()),
            );
        }
        lines.push(dmg_str);
    } else {
        lines.push(locale.monster.no_damage.clone());
    }

    if let Some(blk) = m.block
        && blk > 0
    {
        lines.push(locale.monster.block.replace("{blk}", &blk.to_string()));
    }

    if !m.monster_powers.is_empty() {
        let pwr_str: Vec<String> = m
            .monster_powers
            .iter()
            .map(|p| {
                let base = format!("{}({})", p.name, p.amount);
                if let Some(desc) = power_desc_line(&p.id, p.amount, locale) {
                    format!("{}{}", base, desc)
                } else {
                    base
                }
            })
            .collect();
        let mut line = locale.monster.powers.replace("{list}", &pwr_str.join(" "));
        if m.is_scaling {
            line.push_str(&locale.monster.scaling);
        }
        lines.push(line);
    }

    lines.join("\n")
}

pub(crate) fn build_monsters_section(state: &NormalizedState, locale: &Locale) -> String {
    if state.monsters.is_empty() {
        return String::new();
    }

    let mut lines = vec![
        locale
            .monster
            .section_header
            .replace("{count}", &state.monsters.len().to_string()),
    ];
    for m in &state.monsters {
        lines.push(format_monster(m, locale));
        lines.push(String::new());
    }
    lines.join("\n")
}

pub(crate) fn build_hand_section(state: &NormalizedState, locale: &Locale) -> String {
    if state.hand_cards.is_empty() {
        return String::new();
    }

    let mut lines = vec![
        locale
            .card
            .hand_header
            .replace("{count}", &state.hand_cards.len().to_string()),
    ];
    for c in &state.hand_cards {
        lines.push(format!("  {}", format_card(c, locale)));
    }
    lines.push(String::new());
    lines.join("\n")
}

pub(crate) fn compact_pile(label: &str, cards: &[CardInfo], locale: &Locale) -> String {
    if cards.is_empty() {
        return locale.card.pile_empty.replace("{label}", label);
    }

    let mut name_counts: HashMap<&str, usize> = HashMap::new();
    for c in cards {
        *name_counts.entry(c.name.as_str()).or_insert(0) += 1;
    }

    let mut entries: Vec<String> = name_counts
        .into_iter()
        .map(|(name, count)| {
            if count == 1 {
                name.to_string()
            } else {
                format!("{name}×{count}")
            }
        })
        .collect();
    entries.sort();

    locale
        .card
        .pile
        .replace("{label}", label)
        .replace("{count}", &cards.len().to_string())
        .replace("{cards}", &entries.join(" "))
}

pub(crate) fn format_deck_section(cards: &[CardInfo], locale: &Locale) -> String {
    if cards.is_empty() {
        return String::new();
    }

    struct DeckEntry<'a> {
        card_type: &'a str,
        count: usize,
        cost: i64,
        upgraded: bool,
        name: &'a str,
        description: &'a str,
    }

    let mut aggregated: Vec<DeckEntry> = Vec::new();
    let mut seen: HashMap<(&str, &str), usize> = HashMap::new();
    for c in cards {
        let key = (c.card_type.as_str(), c.id.as_str());
        if let Some(idx) = seen.get(&key) {
            aggregated[*idx].count += 1;
        } else {
            seen.insert(key, aggregated.len());
            aggregated.push(DeckEntry {
                card_type: c.card_type.as_str(),
                count: 1,
                cost: c.cost,
                upgraded: c.upgraded,
                name: c.name.as_str(),
                description: c.description.as_str(),
            });
        }
    }

    let mut by_type: HashMap<&str, Vec<&DeckEntry>> = HashMap::new();
    for entry in &aggregated {
        by_type.entry(entry.card_type).or_default().push(entry);
    }
    for entries in by_type.values_mut() {
        entries.sort_by_key(|e| e.name);
    }

    let type_order = ["ATTACK", "SKILL", "POWER", "CURSE", "STATUS"];

    let mut lines: Vec<String> = Vec::new();
    lines.push(locale.card.deck_header.clone());

    for &t in &type_order {
        if let Some(entries) = by_type.get(t) {
            let type_name = match t {
                "ATTACK" => &locale.i18n.type_attack,
                "SKILL" => &locale.i18n.type_skill,
                "POWER" => &locale.i18n.type_power,
                "CURSE" => &locale.i18n.type_curse,
                "STATUS" => &locale.i18n.type_status,
                _ => t,
            };
            let total: usize = entries.iter().map(|e| e.count).sum();
            lines.push(
                locale
                    .card
                    .type_group
                    .replace("{type}", type_name)
                    .replace("{count}", &total.to_string()),
            );
            for e in entries {
                let desc_str = if e.description.is_empty() {
                    String::new()
                } else {
                    locale
                        .card
                        .with_desc
                        .replace("{desc}", &clean_description(e.description, locale))
                };
                let prefix = if e.upgraded { "+" } else { "" };
                if e.count == 1 {
                    lines.push(format!(
                        "  {prefix}{}({cost}{suffix}){desc_str}",
                        e.name,
                        cost = e.cost,
                        suffix = locale.card.cost_suffix,
                        desc_str = desc_str,
                    ));
                } else {
                    let count_multi = locale
                        .card
                        .card_count_multi
                        .replace("{count}", &e.count.to_string());
                    lines.push(format!(
                        "  {prefix}{}({cost}{suffix}){count_multi}{desc_str}",
                        e.name,
                        cost = e.cost,
                        suffix = locale.card.cost_suffix,
                        count_multi = count_multi,
                        desc_str = desc_str,
                    ));
                }
            }
        }
    }

    lines.join("\n")
}

pub(crate) fn build_relics_potions_section(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines = Vec::new();

    if !state.relics.is_empty() {
        lines.push(locale.sections.relics.clone());
        for r in &state.relics {
            let name = match r.counter {
                Some(c) => format!("{} ({})", r.name, c),
                None => r.name.clone(),
            };
            let desc = match r.counter {
                Some(c) => rewrite_relic_description(&r.id, c, &r.description, locale),
                None => r.description.clone(),
            };
            lines.push(format!("{}：{}", name, clean_description(&desc, locale)));
        }
        lines.push(String::new());
    }

    if !state.potions.is_empty() {
        lines.push(locale.sections.potions.clone());
        for p in &state.potions {
            lines.push(format!(
                "{}：{}",
                p.name,
                clean_description(&p.description, locale)
            ));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

pub(crate) fn build_combat(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec!["[mode: combat]\n".to_string()];

    // === 战斗类型 ===
    lines.push(locale.combat_types.header.clone());

    let room = state.room_type.as_deref();
    let type_name = match room {
        Some("MonsterRoomElite") => &locale.combat_types.type_elite,
        Some("MonsterRoomBoss") => &locale.combat_types.type_boss,
        _ => &locale.combat_types.type_normal,
    };
    lines.push(locale.combat_types.type_line.replace("{type}", type_name));

    let goal = match room {
        Some("MonsterRoomElite") => &locale.combat_types.goal_elite,
        Some("MonsterRoomBoss") => &locale.combat_types.goal_boss,
        _ => &locale.combat_types.goal_normal,
    };
    lines.push(locale.combat_types.primary.replace("{goal}", goal));

    let sub = match room {
        Some("MonsterRoomElite") => &locale.combat_types.sub_elite,
        Some("MonsterRoomBoss") => &locale.combat_types.sub_boss,
        _ => &locale.combat_types.sub_normal,
    };
    lines.push(locale.combat_types.secondary.replace("{sub}", sub));

    let trade = match room {
        Some("MonsterRoomElite") => &locale.combat_types.trade_elite,
        Some("MonsterRoomBoss") => &locale.combat_types.trade_boss,
        _ => &locale.combat_types.trade_normal,
    };
    lines.push(locale.combat_types.trade.replace("{advice}", trade));

    let max_hp = state.max_hp.unwrap_or(75);
    let power = match room {
        Some("MonsterRoomElite") => &locale.combat_types.power_elite,
        Some("MonsterRoomBoss") => &locale.combat_types.power_boss,
        _ if state.incoming_damage > max_hp / 5 => &locale.combat_types.power_normal_high,
        _ => &locale.combat_types.power_normal_low,
    };
    lines.push(locale.combat_types.power_play.replace("{advice}", power));

    let prio: Vec<String> = state
        .monsters
        .iter()
        .map(|m| {
            if m.is_scaling {
                locale.combat_types.prio_scaling.replace("{name}", &m.name)
            } else if m
                .monster_powers
                .iter()
                .any(|p| matches!(p.id.as_str(), "Enrage" | "Thorns" | "Curiosity"))
            {
                locale.combat_types.prio_punish.replace("{name}", &m.name)
            } else if m.can_be_killed {
                locale.combat_types.prio_killable.replace("{name}", &m.name)
            } else {
                locale.combat_types.prio_default.replace("{name}", &m.name)
            }
        })
        .collect();
    lines.push(
        locale
            .combat_types
            .priority
            .replace("{list}", &prio.join("；")),
    );
    lines.push(String::new());

    // === 战斗概况 ===
    lines.push(locale.sections.combat_profile.clone());
    lines.push(combat_profile_line(state, locale));
    lines.push(String::new());

    lines.push(locale.sections.turn_status.clone());
    lines.push(turn_status_line(state, locale));
    lines.push(String::new());

    if !state.potions.is_empty() {
        lines.push(locale.sections.potions.clone());
        for p in &state.potions {
            lines.push(format!(
                "{}：{}",
                p.name,
                clean_description(&p.description, locale)
            ));
        }
        lines.push(String::new());
    }

    if state.danger.wrath_stance {
        lines.push(locale.warnings.wrath_stance.clone());
    }

    if !state.monsters.is_empty() {
        lines.push(build_monsters_section(state, locale));
    }

    if !state.hand_cards.is_empty() {
        lines.push(build_hand_section(state, locale));
    }

    lines.push(compact_pile(
        &locale.card.pile_draw,
        &state.draw_pile,
        locale,
    ));

    lines.push(compact_pile(
        &locale.card.pile_discard,
        &state.discard_pile,
        locale,
    ));

    if !state.exhaust_cards.is_empty() {
        lines.push(compact_pile(
            &locale.card.pile_exhaust,
            &state.exhaust_cards,
            locale,
        ));
    }

    lines.join("\n")
}

pub(crate) fn build_card_reward(state: &NormalizedState, locale: &Locale) -> String {
    let mode = if state.is_boss_card_reward() {
        "[mode: boss_card_reward]\n"
    } else {
        "[mode: card_reward]\n"
    };

    let mut lines: Vec<String> = vec![
        mode.to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    if state.is_boss_card_reward() {
        lines.push(locale.warnings.boss_card_hp_note.clone());
        lines.push(String::new());
    }

    lines.push(format_deck_section(&state.master_cards, locale));

    if !state.card_reward_choices.is_empty() {
        lines.push(locale.sections.card_reward.clone());
        for (i, c) in state.card_reward_choices.iter().enumerate() {
            let label = (b'A' + i as u8) as char;
            lines.push(format!("{label}. {}", format_card(c, locale)));
        }
        if state.skip_available {
            lines.push(locale.warnings.skip.clone());
        }
    }

    lines.join("\n")
}

pub(crate) fn build_rest(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: rest]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    let mut seen_upgradeable = std::collections::HashSet::new();
    let mut upgradeable: Vec<&CardInfo> = state
        .master_cards
        .iter()
        .filter(|c| !c.upgraded && c.card_type != "CURSE" && c.card_type != "STATUS")
        .filter(|c| seen_upgradeable.insert(c.id.as_str()))
        .take(10)
        .collect();
    upgradeable.sort_by_key(|c| c.name.as_str());

    if !upgradeable.is_empty() {
        let names: Vec<String> = upgradeable
            .into_iter()
            .map(|c| format_card(c, locale))
            .collect();
        lines.push(locale.sections.upgrade_targets.clone());
        lines.extend(names);
        lines.push(String::new());
    }

    if !state.rest_options.is_empty() {
        lines.push(locale.sections.options.clone());
        let opts: Vec<String> = state
            .rest_options
            .iter()
            .map(|o| {
                (match o.as_str() {
                    "rest" => &locale.i18n.rest_rest,
                    "smith" => &locale.i18n.rest_smith,
                    "toke" => &locale.i18n.rest_toke,
                    "dig" => &locale.i18n.rest_dig,
                    "lift" => &locale.i18n.rest_lift,
                    "recall" => &locale.i18n.rest_recall,
                    "girya" => &locale.i18n.rest_girya,
                    _ => o.as_str(),
                })
                .to_string()
            })
            .collect();
        lines.push(opts.join("  "));
        lines.push(String::new());
    }

    if let (Some(cur), Some(max)) = (state.current_hp, state.max_hp)
        && max > 0
    {
        let pct = cur as f64 / max as f64;
        if pct < 0.3 {
            lines.push(locale.warnings.low_hp_rest.clone());
        } else if pct > 0.7 {
            lines.push(locale.warnings.high_hp_smith.clone());
        }
    }

    lines.join("\n")
}

pub(crate) fn build_boss_relic(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: boss_relic]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
    ];

    let is_act_end = matches!(state.floor, Some(17) | Some(34));
    if is_act_end {
        lines.push(locale.warnings.boss_relic_hp_note.clone());
    }

    lines.push(String::new());
    lines.push(build_relics_potions_section(state, locale));
    lines.push(format_deck_section(&state.master_cards, locale));

    if !state.boss_relic_choices.is_empty() {
        lines.push(locale.sections.boss_relic.clone());
        for (i, relic) in state.boss_relic_choices.iter().enumerate() {
            let label = (b'A' + i as u8) as char;
            lines.push(format!(
                "{label}. {} — {}",
                relic.name,
                clean_description(&relic.description, locale)
            ));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

pub(crate) fn build_shop(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: shop]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    if !state.master_cards.is_empty() {
        lines.push(format_deck_section(&state.master_cards, locale));
    }

    if !state.shop_cards.is_empty()
        || !state.shop_relics.is_empty()
        || !state.shop_potions.is_empty()
        || state.purge_available
    {
        lines.push(locale.sections.shop.clone());
    }

    if !state.shop_cards.is_empty() {
        lines.push(locale.sections.shop_cards.clone());
        for (i, c) in state.shop_cards.iter().enumerate() {
            let label = (b'A' + i as u8) as char;
            let price = c.price.map_or("?".to_string(), |p| p.to_string());
            lines.push(format!(
                "{label}. {}  ({} gold)",
                format_card(c, locale),
                price
            ));
        }
        lines.push(String::new());
    }

    if !state.shop_relics.is_empty() {
        lines.push(locale.sections.shop_relics.clone());
        for (i, r) in state.shop_relics.iter().enumerate() {
            let label = (b'A' + i as u8) as char;
            let price = r.price.map_or("?".to_string(), |p| p.to_string());
            lines.push(format!(
                "{label}. {} — {}  ({} gold)",
                r.name,
                clean_description(&r.description, locale),
                price
            ));
        }
        lines.push(String::new());
    }

    if !state.shop_potions.is_empty() {
        lines.push(locale.sections.shop_potions.clone());
        for (i, p) in state.shop_potions.iter().enumerate() {
            let label = (b'A' + i as u8) as char;
            let price = p.price.map_or("?".to_string(), |p| p.to_string());
            lines.push(format!(
                "{label}. {} — {}  ({} gold)",
                p.name,
                clean_description(&p.description, locale),
                price
            ));
        }
        lines.push(String::new());
    }

    if state.purge_available {
        lines.push(locale.sections.shop_purge.clone());
        let cost = state
            .purge_cost
            .map_or("unknown".to_string(), |c| c.to_string());
        lines.push(format!("Remove a card for {} gold", cost));
        lines.push(format!("Deck: {}", state.deck_names.join(", ")));
        lines.push(String::new());
    }

    lines.join("\n")
}

pub(crate) fn build_event_choice(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: event_choice]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    if state.event_name.is_some() || state.event_id.is_some() || state.room_type.is_some() {
        lines.push(locale.sections.event.clone());
        if let Some(name) = &state.event_name {
            lines.push(name.clone());
        }
        if let Some(id) = &state.event_id {
            lines.push(locale.warnings.event_id.replace("{id}", id));
        }
        if state.event_name.is_none()
            && state.event_id.is_none()
            && let Some(room_type) = &state.room_type
        {
            lines.push(
                locale
                    .warnings
                    .event_unreadable
                    .replace("{room_type}", room_type),
            );
        }
    }
    if let Some(body) = &state.event_body {
        lines.push(body.clone());
        lines.push(String::new());
    }

    if !state.event_choices.is_empty() {
        lines.push(locale.sections.options.clone());
        for (i, choice) in state.event_choices.iter().enumerate() {
            let label = (b'A' + i as u8) as char;
            lines.push(format!("{label}. {choice}"));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

pub(crate) fn build_generic(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: generic]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    if !state.monsters.is_empty() {
        lines.push(build_monsters_section(state, locale));
    }

    if !state.hand_cards.is_empty() {
        let mut name_counts: HashMap<&str, usize> = HashMap::new();
        for c in &state.hand_cards {
            *name_counts.entry(c.name.as_str()).or_insert(0) += 1;
        }
        let cards: Vec<String> = name_counts
            .into_iter()
            .map(|(name, count)| {
                if count == 1 {
                    name.to_string()
                } else {
                    format!("{name}×{count}")
                }
            })
            .collect();
        lines.push(
            locale
                .sections
                .hand_cards
                .replace("{cards}", &cards.join(" ")),
        );
    }

    lines.join("\n")
}

pub(crate) fn position_label(index: usize, total: usize, locale: &Locale) -> String {
    match (index, total) {
        (0, 1) => &locale.map_position.only,
        (0, 2) => &locale.map_position.left,
        (1, 2) => &locale.map_position.right,
        (0, 3) => &locale.map_position.left,
        (1, 3) => &locale.map_position.middle,
        (2, 3) => &locale.map_position.right,
        (0, _) => &locale.map_position.leftmost,
        _ if index + 1 == total => &locale.map_position.rightmost,
        _ => return from_left_label(index + 1, locale),
    }
    .to_string()
}

pub(crate) fn from_left_label(n: usize, locale: &Locale) -> String {
    locale
        .map_position
        .from_left
        .replace("{ordinal}", &ordinal(n))
        .replace("{n}", &n.to_string())
}

pub(crate) fn ordinal(n: usize) -> String {
    let suffix = if (11..=13).contains(&(n % 100)) {
        "th"
    } else {
        match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{n}{suffix}")
}

pub(crate) struct LabeledPath {
    label: String,
    path: Vec<MapCoord>,
    evaluation: PathEvaluation,
}

pub(crate) fn rank_labeled_paths(
    paths: Vec<(String, Vec<MapCoord>)>,
    state: &NormalizedState,
    shop_visited: bool,
) -> Vec<LabeledPath> {
    let mut ranked: Vec<LabeledPath> = paths
        .into_iter()
        .map(|(label, path)| LabeledPath {
            evaluation: evaluate_path(&path, state, shop_visited),
            label,
            path,
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.evaluation
            .score
            .partial_cmp(&a.evaluation.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.label.cmp(&b.label))
    });
    ranked
}

pub(crate) fn format_candidate_label(index: usize) -> String {
    format!("Candidate {}", index + 1)
}

pub(crate) fn recommendation_features(eval: &PathEvaluation) -> String {
    let metrics = eval.description.metrics;
    let counts = metrics.counts;
    let mut parts = Vec::new();

    match metrics.shop_timing {
        ShopTiming::Early | ShopTiming::Mid | ShopTiming::Late => {
            parts.push(format!("{} shop", metrics.shop_timing.label()));
        }
        ShopTiming::None => {}
    }
    if metrics.rest_before_first_elite {
        parts.push("rest before elite".to_string());
    }
    if counts.elites > 0 {
        parts.push(format!("{} elite{}", counts.elites, plural(counts.elites)));
    }
    if counts.rests >= 2 {
        parts.push(format!("{} rests", counts.rests));
    }
    if counts.events >= 3 {
        parts.push(format!("{} events", counts.events));
    }

    if parts.is_empty() {
        "safe route".to_string()
    } else {
        parts.join(", ")
    }
}

pub(crate) fn compact_route_chain(chain: &str) -> String {
    let parts: Vec<&str> = chain.split('→').collect();
    if parts.len() <= 10 {
        return chain.to_string();
    }

    format!(
        "{}→…→{}",
        parts[..5].join("→"),
        parts[parts.len() - 4..].join("→")
    )
}

pub(crate) fn recommendation_label(base_label: &str, eval: &PathEvaluation) -> String {
    format!(
        "{} — {} — {}",
        base_label,
        compact_route_chain(&eval.description.route_chain),
        recommendation_features(eval)
    )
}

pub(crate) fn build_map_crossroad(
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: map_crossroad]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.next_nodes.clone(),
    ];

    let node_map: HashMap<(i64, i64), &MapCoord> =
        state.map_nodes.iter().map(|n| ((n.x, n.y), n)).collect();

    let current = match (state.map_current_x, state.map_current_y) {
        (Some(cx), Some(cy)) => node_map.get(&(cx, cy)),
        _ => None,
    };

    let Some(current_node) = current else {
        return lines.join("\n");
    };

    let mut valid_children: Vec<&MapCoord> = current_node
        .children
        .iter()
        .filter_map(|(cx, cy)| node_map.get(&(*cx, *cy)).copied())
        .collect();
    valid_children.sort_by_key(|n| (n.x, n.y));

    let total = valid_children.len();
    let mut child_candidates: Vec<(String, Vec<MapCoord>)> = Vec::new();
    for (i, child) in valid_children.iter().enumerate() {
        let label = position_label(i, total, locale);
        let paths = enumerate_paths(child.x, child.y, &state.map_nodes);
        let path = if let Some(path) = paths.first() {
            path.clone()
        } else {
            vec![(*child).clone()]
        };
        let type_name = match child.symbol.as_str() {
            "M" => "monster",
            "E" => "elite",
            "?" => "event",
            "$" => "shop",
            "R" => "rest",
            "T" => "treasure",
            _ => &child.symbol,
        };
        child_candidates.push((format!("({label}) — {}({type_name})", child.symbol), path));
    }

    let ranked = rank_labeled_paths(child_candidates, state, shop_visited);
    for (i, candidate) in ranked.iter().take(MAP_CANDIDATE_LIMIT).enumerate() {
        let eval = &candidate.evaluation;
        let desc = &eval.description;

        let anns: Vec<String> = desc
            .annotations
            .iter()
            .map(|a| {
                if shop_visited && a == "✗ No shop" {
                    "✗ No shop ahead".to_string()
                } else {
                    a.clone()
                }
            })
            .collect();

        let mut details = vec![format!("Ahead: {}", desc.route_chain)];
        if !anns.is_empty() {
            details.push(anns.join("  "));
        }
        details.push(format_evaluation_line(eval));

        lines.push(format!("{}:", format_candidate_label(i)));
        lines.push(format!(
            "  Recommendation label: {}",
            recommendation_label(&candidate.label, eval)
        ));
        lines.push(format!("  {}", details.join("  ")));
    }

    lines.join("\n")
}

pub(crate) fn build_map_suggestion(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: map_suggestion]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.routes.clone(),
    ];

    if state.map_first_node_chosen == Some(true) {
        let paths = match (state.map_current_x, state.map_current_y) {
            (Some(x), Some(y)) => enumerate_paths(x, y, &state.map_nodes),
            _ => vec![],
        };
        let total = paths.len();
        let labeled_paths: Vec<(String, Vec<MapCoord>)> = paths
            .into_iter()
            .enumerate()
            .map(|(i, path)| {
                let pos = position_label(i, total, locale);
                (format!("Route {} ({pos})", i + 1), path)
            })
            .collect();
        let ranked = rank_labeled_paths(labeled_paths, state, false);
        for (i, candidate) in ranked.iter().take(MAP_CANDIDATE_LIMIT).enumerate() {
            let path = &candidate.path;
            let route: Vec<String> = path
                .iter()
                .map(|n| format!("{}({},{})", n.symbol, n.x, n.y))
                .collect();
            let eval = &candidate.evaluation;
            let desc = &eval.description;
            lines.push(format!("{}:", format_candidate_label(i)));
            lines.push(format!(
                "  Recommendation label: {}",
                recommendation_label(&candidate.label, eval)
            ));
            lines.push(format!("  {}  [{}]", route.join("→"), desc.counts));
            let mut ann_line = format!("  {}", desc.route_chain);
            if !desc.annotations.is_empty() {
                ann_line.push_str(&format!("  {}", desc.annotations.join("  ")));
            }
            ann_line.push_str(&format!("  {}", format_evaluation_line(eval)));
            lines.push(ann_line);
        }
    } else {
        let mut roots = enumerate_paths_from_roots(&state.map_nodes);
        roots.sort_by_key(|r| r.root.x);
        let root_total = roots.len();
        let mut labeled_paths: Vec<(String, Vec<MapCoord>)> = Vec::new();
        for (ri, r) in roots.iter().enumerate() {
            let pos = position_label(ri, root_total, locale);
            for path in &r.paths {
                labeled_paths.push((format!("Root {} ({pos})", ri + 1), path.clone()));
            }
        }
        let ranked = rank_labeled_paths(labeled_paths, state, false);
        for (i, candidate) in ranked.iter().take(MAP_CANDIDATE_LIMIT).enumerate() {
            let route: Vec<String> = candidate
                .path
                .iter()
                .map(|n| format!("{}({},{})", n.symbol, n.x, n.y))
                .collect();
            let eval = &candidate.evaluation;
            let desc = &eval.description;
            lines.push(format!("{}:", format_candidate_label(i)));
            lines.push(format!(
                "  Recommendation label: {}",
                recommendation_label(&candidate.label, eval)
            ));
            lines.push(format!("  {}  [{}]", route.join("→"), desc.counts));
            let mut ann_line = format!("  {}", desc.route_chain);
            if !desc.annotations.is_empty() {
                ann_line.push_str(&format!("  {}", desc.annotations.join("  ")));
            }
            ann_line.push_str(&format!("  {}", format_evaluation_line(eval)));
            lines.push(ann_line);
        }
    }

    lines.join("\n")
}

pub(crate) fn build_hand_select(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: hand_select]\n".to_string(),
        locale.sections.hand_select.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    // Purpose: tell the LLM why cards are being selected
    if let Some(ref action) = state.current_action {
        let action_desc = match action.as_str() {
            "ExhaustAction" => "Exhaust a card".to_string(),
            "DiscardAction" => "Discard a card".to_string(),
            "PutOnDeckAction" => "Put a card on top of your draw pile".to_string(),
            _ => action.to_string(),
        };
        if let Some(ref card) = state.card_in_play {
            lines.push(format!("Card playing: {}", format_card(card, locale)));
        }
        lines.push(format!("Purpose: {action_desc}"));
    }
    if let Some(max) = state.hand_select_max_cards {
        lines.push(format!(
            "Max: {max}  Can skip: {}",
            if state.hand_select_can_pick_zero {
                "yes"
            } else {
                "no"
            }
        ));
    }

    // Show already selected cards
    if !state.hand_select_selected.is_empty() {
        let selected: Vec<String> = state
            .hand_select_selected
            .iter()
            .map(|c| c.name.clone())
            .collect();
        lines.push(format!(
            "{} [{}]",
            locale.sections.selected_cards,
            selected.join(", ")
        ));
    }

    // Available cards
    lines.push(String::new());
    lines.push(locale.sections.hand_select_available.clone());
    for (i, card) in state.hand.iter().enumerate() {
        lines.push(format!("  {i}. {}", format_card(card, locale)));
    }

    lines.join("\n")
}

pub(crate) fn build_grid_select(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: grid_select]\n".to_string(),
        locale.sections.grid_select.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    // Purpose
    let purpose = if state.grid_for_upgrade {
        locale.i18n.grid_upgrade.as_str()
    } else if state.grid_for_transform {
        locale.i18n.grid_transform.as_str()
    } else if state.grid_for_purge {
        locale.i18n.grid_purge.as_str()
    } else {
        locale.i18n.grid_other.as_str()
    };
    lines.push(purpose.to_string());
    if let Some(num) = state.grid_num_cards {
        lines.push(format!("Select {} card(s).", num));
    }
    lines.push(String::new());

    // Available cards
    lines.push(locale.sections.hand_select_available.clone());
    let cards = if !state.grid_cards.is_empty() {
        &state.grid_cards
    } else {
        &state.hand
    };
    for (i, card) in cards.iter().enumerate() {
        lines.push(format!("  {i}. {}", format_card(card, locale)));
    }

    lines.join("\n")
}

pub fn build_prompt(state: &NormalizedState, locale: &Locale, shop_visited: bool) -> String {
    match state.screen_type.as_deref() {
        Some("CARD_REWARD") => build_card_reward(state, locale),
        Some("BOSS_REWARD") => build_boss_relic(state, locale),
        Some("REST") => build_rest(state, locale),
        Some("EVENT") => build_event_choice(state, locale),
        Some("SHOP_SCREEN") => build_shop(state, locale),
        Some("MAP") if state.map_first_node_chosen == Some(true) => {
            build_map_crossroad(state, locale, shop_visited)
        }
        Some("MAP") => build_map_suggestion(state, locale),
        Some("HAND_SELECT") => build_hand_select(state, locale),
        Some("GRID") => build_grid_select(state, locale),
        _ if !state.monsters.is_empty() => build_combat(state, locale),
        _ => build_generic(state, locale),
    }
}

pub(crate) fn format_evaluation_line(eval: &PathEvaluation) -> String {
    let mut parts = vec![format!("Score:{:.0}", eval.score)];
    if !eval.pros.is_empty() {
        parts.push(format!("+{}", eval.pros.join(", ")));
    }
    if !eval.cons.is_empty() {
        parts.push(format!("-{}", eval.cons.join(", ")));
    }
    parts.join("  ")
}
