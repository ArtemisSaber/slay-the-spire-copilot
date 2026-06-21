use crate::locales::Locale;
use crate::state::{CardInfo, DangerLevel, MapCoord, MonsterInfo, NormalizedState};
use std::collections::HashMap;

fn danger_prefix(state: &NormalizedState, locale: &Locale) -> String {
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

fn status_line(state: &NormalizedState, locale: &Locale) -> String {
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

    // Player powers
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

    // Relics & potions
    if !state.relics.is_empty() {
        let names: Vec<&str> = state.relics.iter().map(|r| r.name.as_str()).collect();
        parts.push(locale.status.relics.replace("{list}", &names.join(" ")));
    }
    if !state.potions.is_empty() {
        let names: Vec<&str> = state.potions.iter().map(|p| p.name.as_str()).collect();
        parts.push(locale.status.potions.replace("{list}", &names.join(" ")));
    }

    // Incoming damage
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

fn clean_description(raw: &str, locale: &Locale) -> String {
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

fn format_card(c: &CardInfo, locale: &Locale) -> String {
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

fn format_monster(m: &MonsterInfo, locale: &Locale) -> String {
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
            .map(|p| format!("{}({})", p.name, p.amount))
            .collect();
        let mut line = locale.monster.powers.replace("{list}", &pwr_str.join(" "));
        if m.is_scaling {
            line.push_str(&locale.monster.scaling);
        }
        lines.push(line);
    }

    lines.join("\n")
}

fn build_monsters_section(state: &NormalizedState, locale: &Locale) -> String {
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

fn build_hand_section(state: &NormalizedState, locale: &Locale) -> String {
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

fn compact_pile(label: &str, cards: &[CardInfo], locale: &Locale) -> String {
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

fn format_deck_section(cards: &[CardInfo], locale: &Locale) -> String {
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

    lines.push(String::new());
    lines.join("\n")
}

fn build_relics_potions_section(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines = Vec::new();

    if !state.relics.is_empty() {
        lines.push(locale.sections.relics.clone());
        for r in &state.relics {
            lines.push(format!(
                "{}：{}",
                r.name,
                clean_description(&r.description, locale)
            ));
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

fn build_combat(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.task.clone(),
        locale.tasks.combat_entry.clone(),
        String::new(),
    ];

    if state.danger.no_block_against_hit {
        lines.push(locale.warnings.no_block.clone());
    }
    if state.danger.wrath_stance {
        lines.push(locale.warnings.wrath_stance.clone());
    }

    // Monsters
    if !state.monsters.is_empty() {
        lines.push(build_monsters_section(state, locale));
    }

    // Hand cards with descriptions
    if !state.hand_cards.is_empty() {
        lines.push(build_hand_section(state, locale));
    }

    // Draw pile
    lines.push(compact_pile(
        &locale.card.pile_draw,
        &state.draw_pile,
        locale,
    ));

    // Discard pile
    lines.push(compact_pile(
        &locale.card.pile_discard,
        &state.discard_pile,
        locale,
    ));

    // Exhaust pile (only if non-empty)
    if !state.exhaust_cards.is_empty() {
        lines.push(compact_pile(
            &locale.card.pile_exhaust,
            &state.exhaust_cards,
            locale,
        ));
    }

    lines.push(format_line(locale).to_string());
    lines.join("\n")
}

fn build_card_reward(state: &NormalizedState, locale: &Locale) -> String {
    let task = if state.is_boss_card_reward() {
        &locale.tasks.boss_card_reward
    } else {
        &locale.tasks.card_reward
    };

    let mut lines: Vec<String> = vec![
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.task.clone(),
        task.clone(),
        String::new(),
    ];

    if state.is_boss_card_reward() {
        lines.push(locale.warnings.boss_card_hp_note.clone());
        lines.push(String::new());
    }

    lines.push(format_deck_section(&state.master_cards, locale));

    // Reward choices
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

    lines.push(format_line(locale).to_string());
    lines.join("\n")
}

fn build_rest(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.task.clone(),
        locale.tasks.rest.clone(),
        String::new(),
    ];

    // Upgradeable cards (unupgraded cards in master deck)
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

    // HP-based advice
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

    lines.push(format_line(locale).to_string());
    lines.join("\n")
}

fn build_boss_relic(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        locale.sections.current_state.clone(),
        status_line(state, locale),
    ];

    let is_act_end = matches!(state.floor, Some(17) | Some(34));
    if is_act_end {
        lines.push(locale.warnings.boss_relic_hp_note.clone());
    }

    lines.push(String::new());
    lines.push(build_relics_potions_section(state, locale));
    lines.push(locale.sections.task.clone());
    lines.push(locale.tasks.boss_relic.clone());
    lines.push(String::new());
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

    lines.push(format_line(locale).to_string());
    lines.join("\n")
}

fn build_event_choice(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.task.clone(),
        locale.tasks.event_choice.clone(),
        String::new(),
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

    lines.push(format_line(locale).to_string());
    lines.join("\n")
}

fn build_generic(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.task.clone(),
        locale.tasks.generic.clone(),
        String::new(),
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

    lines.push(format_line(locale).to_string());
    lines.join("\n")
}

fn format_line(locale: &Locale) -> &str {
    &locale.format_footer
}

pub fn build_prompt(state: &NormalizedState, locale: &Locale) -> String {
    match state.screen_type.as_deref() {
        Some("CARD_REWARD") => build_card_reward(state, locale),
        Some("BOSS_REWARD") => build_boss_relic(state, locale),
        Some("REST") => build_rest(state, locale),
        Some("EVENT") => build_event_choice(state, locale),
        _ if !state.monsters.is_empty() => build_combat(state, locale),
        _ => build_generic(state, locale),
    }
}

pub fn enumerate_paths(start_x: i64, start_y: i64, nodes: &[MapCoord]) -> Vec<Vec<MapCoord>> {
    let node_map: HashMap<(i64, i64), &MapCoord> = nodes.iter().map(|n| ((n.x, n.y), n)).collect();

    let start = node_map.get(&(start_x, start_y));

    let Some(start_node) = start else {
        return vec![];
    };

    let mut paths = Vec::new();
    let mut stack = vec![(vec![(*start_node).clone()], *start_node)];

    while let Some((path, node)) = stack.pop() {
        if node.children.is_empty() {
            paths.push(path);
            continue;
        }
        let mut found = false;
        for (cx, cy) in node.children.iter().rev() {
            if let Some(child) = node_map.get(&(*cx, *cy)) {
                found = true;
                let mut new_path = path.clone();
                new_path.push((*child).clone());
                stack.push((new_path, child));
            }
        }
        if !found {
            paths.push(path);
        }
    }

    paths.sort_by(|a, b| {
        for (na, nb) in a.iter().zip(b.iter()) {
            let cx = na.x.cmp(&nb.x);
            if cx != std::cmp::Ordering::Equal {
                return cx;
            }
            let cy = na.y.cmp(&nb.y);
            if cy != std::cmp::Ordering::Equal {
                return cy;
            }
        }
        a.len().cmp(&b.len())
    });

    paths
}

pub fn summarize_path(path: &[MapCoord]) -> String {
    let mut monsters = 0;
    let mut elites = 0;
    let mut events = 0;
    let mut shops = 0;
    let mut rests = 0;
    let mut treasures = 0;
    for n in path {
        match n.symbol.as_str() {
            "M" => monsters += 1,
            "E" => elites += 1,
            "?" => events += 1,
            "$" => shops += 1,
            "R" => rests += 1,
            "T" => treasures += 1,
            _ => {}
        }
    }
    let mut parts = Vec::new();
    if monsters > 0 {
        parts.push(format!("Monsters:{monsters}"));
    }
    if elites > 0 {
        parts.push(format!("Elites:{elites}"));
    }
    if events > 0 {
        parts.push(format!("Events:{events}"));
    }
    if shops > 0 {
        parts.push(format!("Shops:{shops}"));
    }
    if rests > 0 {
        parts.push(format!("Rests:{rests}"));
    }
    if treasures > 0 {
        parts.push(format!("Treasures:{treasures}"));
    }
    parts.join("  ")
}

pub struct PathDescription {
    pub counts: String,
    pub route_chain: String,
    pub annotations: Vec<String>,
}

fn act_from_floor(floor: i64) -> u8 {
    match floor {
        1..=16 => 1,
        17..=33 => 2,
        34..=50 => 3,
        _ => 1,
    }
}

fn risk_modifier(symbol: &str, act: u8) -> f64 {
    match symbol {
        "E" => 10.0,
        "M" => match act {
            1 => 2.0,
            2 => 4.0,
            3 => 3.0,
            _ => 2.0,
        },
        "?" => match act {
            1 => 1.0,
            2 => 2.0,
            3 => 1.5,
            _ => 1.0,
        },
        _ => 0.0,
    }
}

pub fn describe_path(path: &[MapCoord], floor: i64) -> PathDescription {
    let act = act_from_floor(floor);
    let counts = summarize_path(path);
    let route_chain = path
        .iter()
        .map(|n| n.symbol.as_str())
        .collect::<Vec<_>>()
        .join("→");

    let mut annotations = Vec::new();

    if let Some(ei) = path.iter().position(|n| n.symbol == "E") {
        if path[..ei].iter().any(|n| n.symbol == "R") {
            annotations.push("✓ Rest before first Elite".to_string());
        }
    }

    let r_indices: Vec<usize> = path
        .iter()
        .enumerate()
        .filter(|(_, n)| n.symbol == "R")
        .map(|(i, _)| i)
        .collect();

    let mut segment_starts = Vec::new();
    let mut segment_ends = Vec::new();

    if let Some(&first_r) = r_indices.first() {
        if first_r > 0 {
            segment_starts.push(0);
            segment_ends.push(first_r);
        }
    } else {
        segment_starts.push(0);
        segment_ends.push(path.len());
    }

    for pair in r_indices.windows(2) {
        let start = pair[0] + 1;
        let end = pair[1];
        if start < end {
            segment_starts.push(start);
            segment_ends.push(end);
        }
    }

    for (start, end) in segment_starts.iter().zip(segment_ends.iter()) {
        let seg = &path[*start..*end];
        let elite_count = seg.iter().filter(|n| n.symbol == "E").count();
        if elite_count >= 2 {
            annotations.push("⚠ Double Elite — no Rest between".to_string());
        }

        for (j, n) in seg.iter().enumerate() {
            if n.symbol == "E" {
                let gap: f64 = seg[j..].iter().map(|m| risk_modifier(&m.symbol, act)).sum();
                if gap > 15.0 {
                    annotations.push(format!("⚠ max E→R gap: {:.0} risk", gap));
                    break;
                }
            }
        }
    }

    let shop_idx = path.iter().position(|n| n.symbol == "$");
    if let Some(si) = shop_idx {
        let pos = si as f64 / path.len() as f64;
        let label = if pos < 0.33 {
            "early"
        } else if pos < 0.66 {
            "mid"
        } else {
            "late"
        };
        annotations.push(format!("$ Shop ({label})"));
    } else {
        annotations.push("✗ No shop".to_string());
    }

    PathDescription {
        counts,
        route_chain,
        annotations,
    }
}

pub struct RootPaths {
    pub root: MapCoord,
    pub paths: Vec<Vec<MapCoord>>,
}

pub fn enumerate_paths_from_roots(nodes: &[MapCoord]) -> Vec<RootPaths> {
    let mut result = Vec::new();
    let roots: Vec<&MapCoord> = nodes.iter().filter(|n| n.y == 0).collect();

    for root in roots {
        let paths = enumerate_paths(root.x, root.y, nodes);
        result.push(RootPaths {
            root: (*root).clone(),
            paths,
        });
    }

    result
}

#[cfg(test)]
#[path = "tests/prompt_tests.rs"]
mod tests;
