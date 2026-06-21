use crate::locales::Locale;
use crate::state::{CardInfo, DangerLevel, MapCoord, MonsterInfo, NormalizedState};
use std::cmp::Ordering;
use std::collections::HashMap;

const MAP_CANDIDATE_LIMIT: usize = 5;

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

fn position_label(index: usize, total: usize, locale: &Locale) -> String {
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

fn from_left_label(n: usize, locale: &Locale) -> String {
    locale
        .map_position
        .from_left
        .replace("{ordinal}", &ordinal(n))
        .replace("{n}", &n.to_string())
}

fn ordinal(n: usize) -> String {
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

struct LabeledPath {
    label: String,
    path: Vec<MapCoord>,
    evaluation: PathEvaluation,
}

fn rank_labeled_paths(
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

fn format_candidate_label(index: usize) -> String {
    format!("Candidate {}", index + 1)
}

fn recommendation_features(eval: &PathEvaluation) -> String {
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

fn compact_route_chain(chain: &str) -> String {
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

fn recommendation_label(base_label: &str, eval: &PathEvaluation) -> String {
    format!(
        "{} — {} — {}",
        base_label,
        compact_route_chain(&eval.description.route_chain),
        recommendation_features(eval)
    )
}

fn build_map_crossroad(state: &NormalizedState, locale: &Locale, shop_visited: bool) -> String {
    let mut lines: Vec<String> = vec![
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.task.clone(),
        locale.tasks.map_crossroad.clone(),
        String::new(),
        locale.sections.next_nodes.clone(),
    ];

    let node_map: HashMap<(i64, i64), &MapCoord> =
        state.map_nodes.iter().map(|n| ((n.x, n.y), n)).collect();

    let current = match (state.map_current_x, state.map_current_y) {
        (Some(cx), Some(cy)) => node_map.get(&(cx, cy)),
        _ => None,
    };

    let Some(current_node) = current else {
        lines.push(format_line(locale).to_string());
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

    lines.push(String::new());
    lines.push(format_line(locale).to_string());
    lines.join("\n")
}

fn build_map_suggestion(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
        locale.sections.task.clone(),
        locale.tasks.map_suggestion.clone(),
        String::new(),
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

    lines.push(String::new());
    lines.push(format_line(locale).to_string());
    lines.join("\n")
}

pub fn build_prompt(state: &NormalizedState, locale: &Locale, shop_visited: bool) -> String {
    match state.screen_type.as_deref() {
        Some("CARD_REWARD") => build_card_reward(state, locale),
        Some("BOSS_REWARD") => build_boss_relic(state, locale),
        Some("REST") => build_rest(state, locale),
        Some("EVENT") => build_event_choice(state, locale),
        Some("MAP") if state.map_first_node_chosen == Some(true) => {
            build_map_crossroad(state, locale, shop_visited)
        }
        Some("MAP") => build_map_suggestion(state, locale),
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
    let counts = count_path_nodes(path);
    let mut parts = Vec::new();
    if counts.monsters > 0 {
        parts.push(format!("Monsters:{}", counts.monsters));
    }
    if counts.elites > 0 {
        parts.push(format!("Elites:{}", counts.elites));
    }
    if counts.events > 0 {
        parts.push(format!("Events:{}", counts.events));
    }
    if counts.shops > 0 {
        parts.push(format!("Shops:{}", counts.shops));
    }
    if counts.rests > 0 {
        parts.push(format!("Rests:{}", counts.rests));
    }
    if counts.treasures > 0 {
        parts.push(format!("Treasures:{}", counts.treasures));
    }
    parts.join("  ")
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PathCounts {
    pub monsters: usize,
    pub elites: usize,
    pub events: usize,
    pub shops: usize,
    pub rests: usize,
    pub treasures: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ShopTiming {
    #[default]
    None,
    Early,
    Mid,
    Late,
}

impl ShopTiming {
    fn label(self) -> &'static str {
        match self {
            ShopTiming::None => "none",
            ShopTiming::Early => "early",
            ShopTiming::Mid => "mid",
            ShopTiming::Late => "late",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathMetrics {
    pub counts: PathCounts,
    pub shop_timing: ShopTiming,
    pub rest_before_first_elite: bool,
    pub double_elite_without_rest: bool,
    pub max_elite_to_rest_risk: f64,
}

pub struct PathDescription {
    pub counts: String,
    pub route_chain: String,
    pub annotations: Vec<String>,
    pub metrics: PathMetrics,
}

pub struct PathEvaluation {
    pub description: PathDescription,
    pub score: f64,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
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

fn count_path_nodes(path: &[MapCoord]) -> PathCounts {
    let mut counts = PathCounts::default();
    for n in path {
        match n.symbol.as_str() {
            "M" => counts.monsters += 1,
            "E" => counts.elites += 1,
            "?" => counts.events += 1,
            "$" => counts.shops += 1,
            "R" => counts.rests += 1,
            "T" => counts.treasures += 1,
            _ => {}
        }
    }
    counts
}

fn segment_bounds_between_rests(path: &[MapCoord]) -> Vec<(usize, usize)> {
    let r_indices: Vec<usize> = path
        .iter()
        .enumerate()
        .filter(|(_, n)| n.symbol == "R")
        .map(|(i, _)| i)
        .collect();

    let mut bounds = Vec::new();
    let mut start = 0;
    for rest in r_indices {
        if start < rest {
            bounds.push((start, rest));
        }
        start = rest + 1;
    }
    if start < path.len() {
        bounds.push((start, path.len()));
    }
    bounds
}

fn path_metrics(path: &[MapCoord], floor: i64) -> PathMetrics {
    let act = act_from_floor(floor);
    let counts = count_path_nodes(path);

    let rest_before_first_elite = path
        .iter()
        .position(|n| n.symbol == "E")
        .map(|ei| path[..ei].iter().any(|n| n.symbol == "R"))
        .unwrap_or(false);

    let mut double_elite_without_rest = false;
    let mut max_elite_to_rest_risk = 0.0;
    for (start, end) in segment_bounds_between_rests(path) {
        let seg = &path[start..end];
        let elite_count = seg.iter().filter(|n| n.symbol == "E").count();
        if elite_count >= 2 {
            double_elite_without_rest = true;
        }

        for (j, n) in seg.iter().enumerate() {
            if n.symbol == "E" {
                let gap: f64 = seg[j..].iter().map(|m| risk_modifier(&m.symbol, act)).sum();
                if gap > max_elite_to_rest_risk {
                    max_elite_to_rest_risk = gap;
                }
            }
        }
    }

    let shop_timing = path
        .iter()
        .position(|n| n.symbol == "$")
        .map(|si| {
            let pos = si as f64 / path.len().max(1) as f64;
            if pos < 0.33 {
                ShopTiming::Early
            } else if pos < 0.66 {
                ShopTiming::Mid
            } else {
                ShopTiming::Late
            }
        })
        .unwrap_or(ShopTiming::None);

    PathMetrics {
        counts,
        shop_timing,
        rest_before_first_elite,
        double_elite_without_rest,
        max_elite_to_rest_risk,
    }
}

pub fn describe_path(path: &[MapCoord], floor: i64) -> PathDescription {
    let metrics = path_metrics(path, floor);
    let counts = summarize_path(path);
    let route_chain = path
        .iter()
        .map(|n| n.symbol.as_str())
        .collect::<Vec<_>>()
        .join("→");

    let mut annotations = Vec::new();

    if metrics.rest_before_first_elite {
        annotations.push("✓ Rest before first Elite".to_string());
    }

    if metrics.double_elite_without_rest {
        annotations.push("⚠ Double Elite — no Rest between".to_string());
    }

    if metrics.max_elite_to_rest_risk > 15.0 {
        annotations.push(format!(
            "⚠ max E→R gap: {:.0} risk",
            metrics.max_elite_to_rest_risk
        ));
    }

    match metrics.shop_timing {
        ShopTiming::Early | ShopTiming::Mid | ShopTiming::Late => {
            annotations.push(format!("$ Shop ({})", metrics.shop_timing.label()));
        }
        ShopTiming::None => annotations.push("✗ No shop".to_string()),
    }

    PathDescription {
        counts,
        route_chain,
        annotations,
        metrics,
    }
}

fn format_evaluation_line(eval: &PathEvaluation) -> String {
    let mut parts = vec![format!("Score:{:.0}", eval.score)];
    if !eval.pros.is_empty() {
        parts.push(format!("+{}", eval.pros.join(", ")));
    }
    if !eval.cons.is_empty() {
        parts.push(format!("-{}", eval.cons.join(", ")));
    }
    parts.join("  ")
}

fn hp_ratio(state: &NormalizedState) -> f64 {
    match (state.current_hp, state.max_hp) {
        (Some(cur), Some(max)) if max > 0 => cur as f64 / max as f64,
        _ => 0.75,
    }
}

fn score_shop(timing: ShopTiming, shop_count: usize, gold: i64, shop_visited: bool) -> f64 {
    let base = match gold {
        g if g >= 180 => match timing {
            ShopTiming::Early => 12.0,
            ShopTiming::Mid => 8.0,
            ShopTiming::Late => 4.0,
            ShopTiming::None => -5.0,
        },
        g if g >= 100 => match timing {
            ShopTiming::Early => 9.0,
            ShopTiming::Mid => 6.0,
            ShopTiming::Late => 3.0,
            ShopTiming::None if shop_visited => -1.0,
            ShopTiming::None => -3.0,
        },
        g if g >= 60 => match timing {
            ShopTiming::Early => 4.0,
            ShopTiming::Mid => 3.0,
            ShopTiming::Late => 1.0,
            ShopTiming::None => 0.0,
        },
        _ => match timing {
            ShopTiming::None => 1.0,
            ShopTiming::Late => 0.0,
            ShopTiming::Early | ShopTiming::Mid => -1.0,
        },
    };

    let extra_shop_bonus = shop_count.saturating_sub(1) as f64
        * if gold >= 250 {
            2.0
        } else if gold >= 120 {
            0.5
        } else {
            -1.0
        };

    base + extra_shop_bonus
}

pub fn evaluate_path(
    path: &[MapCoord],
    state: &NormalizedState,
    shop_visited: bool,
) -> PathEvaluation {
    let floor = state.floor.unwrap_or(1);
    let description = describe_path(path, floor);
    let metrics = description.metrics;
    let counts = metrics.counts;
    let hp = hp_ratio(state);
    let gold = state.gold.unwrap_or(0);
    let act = act_from_floor(floor);

    let mut score = 50.0;
    let elite_value = match act {
        1 => 12.0,
        2 => 10.0,
        3 => 8.0,
        _ => 10.0,
    } + if hp >= 0.75 {
        3.0
    } else if hp < 0.45 {
        -8.0
    } else {
        0.0
    };

    score += counts.elites as f64 * elite_value;
    score += counts.treasures as f64 * 5.0;
    score += counts.events as f64 * if hp < 0.45 { 3.0 } else { 2.0 };
    score += counts.rests as f64
        * if hp < 0.45 {
            6.0
        } else if hp < 0.7 {
            4.0
        } else {
            2.0
        };
    score += counts.monsters as f64 * if hp < 0.45 { -1.5 } else { 0.8 };
    score += score_shop(metrics.shop_timing, counts.shops, gold, shop_visited);

    if metrics.rest_before_first_elite {
        score += if hp < 0.6 { 10.0 } else { 6.0 };
    } else if counts.elites > 0 && hp < 0.6 {
        score -= 8.0;
    }

    if metrics.double_elite_without_rest {
        score -= if hp < 0.6 { 30.0 } else { 20.0 };
    }

    let risk_tolerance = if hp >= 0.75 {
        24.0
    } else if hp >= 0.55 {
        18.0
    } else {
        12.0
    };
    if metrics.max_elite_to_rest_risk > risk_tolerance {
        score -= (metrics.max_elite_to_rest_risk - risk_tolerance) * 1.2;
    }

    if counts.elites > 0 && counts.rests == 0 {
        score -= if hp < 0.6 { 14.0 } else { 6.0 };
    }

    let mut pros = Vec::new();
    let mut cons = Vec::new();

    if counts.elites > 0 {
        pros.push(format!(
            "{} elite reward{}",
            counts.elites,
            plural(counts.elites)
        ));
    }
    if metrics.rest_before_first_elite {
        pros.push("rest before first elite".to_string());
    }
    match metrics.shop_timing {
        ShopTiming::Early | ShopTiming::Mid | ShopTiming::Late => {
            pros.push(format!("{} shop", metrics.shop_timing.label()));
        }
        ShopTiming::None if !shop_visited && gold >= 100 => {
            cons.push("no shop for current gold".to_string());
        }
        ShopTiming::None if !shop_visited => cons.push("no shop".to_string()),
        ShopTiming::None => cons.push("no shop ahead".to_string()),
    }
    if counts.events >= 3 {
        pros.push(format!("{} events", counts.events));
    }
    if counts.rests >= 2 {
        pros.push(format!("{} rests", counts.rests));
    }
    if counts.treasures > 0 {
        pros.push("treasure".to_string());
    }

    if metrics.double_elite_without_rest {
        cons.push("double elite without rest".to_string());
    }
    if metrics.max_elite_to_rest_risk > 15.0 {
        cons.push(format!(
            "high E->R risk {:.0}",
            metrics.max_elite_to_rest_risk
        ));
    }
    if counts.elites > 0 && !metrics.rest_before_first_elite {
        cons.push("no rest before first elite".to_string());
    }
    if hp < 0.45 && counts.monsters >= 6 {
        cons.push("many hallway fights at low HP".to_string());
    }

    PathEvaluation {
        description,
        score,
        pros,
        cons,
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
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
