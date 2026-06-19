use crate::i18n::{self, I18n};
use crate::state::{CardInfo, DangerLevel, MonsterInfo, NormalizedState};
use std::collections::HashMap;

fn danger_prefix(state: &NormalizedState) -> String {
    let mut reasons: Vec<&str> = Vec::new();

    if state.danger.incoming_lethal {
        reasons.push("致命伤害");
    }
    if state.danger.hp_critical {
        reasons.push("血量危急");
    }
    if state.danger.wrath_stance && state.danger.any_monster_attacking {
        reasons.push("愤怒姿态");
    }
    if state.danger.no_block_against_hit {
        reasons.push("无格挡");
    }

    match state.danger.level {
        DangerLevel::Danger => {
            if reasons.is_empty() {
                "危险！".to_string()
            } else {
                format!("危险！{}！", reasons.join("，"))
            }
        }
        DangerLevel::Caution => "小心行事。".to_string(),
        DangerLevel::Safe => "形势不错。".to_string(),
    }
}

fn status_line(state: &NormalizedState) -> String {
    let mut parts: Vec<String> = Vec::new();

    parts.push(danger_prefix(state));

    if let Some(ref c) = state.character {
        parts.push(format!("角色：{}", i18n::translate_class(c)));
    }
    if let Some(f) = state.floor {
        parts.push(format!("层数：{f}"));
    }
    if let (Some(cur), Some(max)) = (state.current_hp, state.max_hp) {
        let pct = if max > 0 {
            (cur as f64 / max as f64 * 100.0) as i64
        } else {
            0
        };
        parts.push(format!("血量：{cur}/{max}({pct}%)"));
    }
    if let Some(b) = state.block {
        parts.push(format!("格挡：{b}"));
    }
    if let Some(e) = state.energy {
        parts.push(format!("能量：{e}"));
    }
    if let Some(g) = state.gold {
        parts.push(format!("金币：{g}"));
    }

    // Player powers
    if !state.powers.is_empty() {
        let powers_str: Vec<String> = state
            .powers
            .iter()
            .map(|p| format!("{}({})", p.name, p.amount))
            .collect();
        parts.push(format!("能力：{}", powers_str.join(" ")));
    }

    // Relics & potions
    if !state.relics.is_empty() {
        parts.push(format!("遗物：{}", state.relics.join(" ")));
    }
    if !state.potions.is_empty() {
        parts.push(format!("药水：{}", state.potions.join(" ")));
    }

    // Incoming damage
    if state.incoming_damage > 0 {
        parts.push(format!(
            "伤害合计{}{}",
            state.incoming_damage,
            if state.block.unwrap_or(0) > 0 && state.incoming_damage > state.block.unwrap_or(0) {
                "（需格挡！）"
            } else {
                ""
            }
        ));
    }

    parts.join("  ")
}

fn resolve_description(id: &str, upgraded: bool, raw: &str, i18n_data: &I18n) -> String {
    let mut result = raw.to_string();
    if let Some(val) = i18n_data.card_values.get(id) {
        let d = if upgraded { val.du } else { val.d };
        let b = if upgraded { val.bu } else { val.b };
        let m = if upgraded { val.mu } else { val.m };
        if d > 0 {
            result = result.replace("!D!", &d.to_string());
        }
        if b > 0 {
            result = result.replace("!B!", &b.to_string());
        }
        if m > 0 {
            result = result.replace("!M!", &m.to_string());
        }
    }
    result = result.replace(" NL ", "\n");
    result = result.replace('*', "");
    for (token, replacement) in &[
        ("[R]", "能量"),
        ("[G]", "能量"),
        ("[B]", "能量"),
        ("[W]", "能量"),
    ] {
        result = result.replace(token, replacement);
    }
    for n in (2..=10).rev() {
        let pattern: String = (0..n).map(|_| "能量").collect::<Vec<_>>().join(" ");
        let replacement = format!("能量{n}");
        result = result.replace(&pattern, &replacement);
    }
    result
}

fn format_card(c: &CardInfo, i18n_data: &I18n) -> String {
    let display_name = i18n_data.card(&c.id).unwrap_or(&c.name);
    let desc = i18n_data
        .card_desc(&c.id)
        .map(|raw| {
            format!(
                " — {}",
                resolve_description(&c.id, c.upgraded, raw, i18n_data)
            )
        })
        .unwrap_or_default();
    format!(
        "{up}{name}({cost}费/{ctype}){desc}",
        up = if c.upgraded { "+" } else { "" },
        name = display_name,
        cost = c.cost,
        ctype = i18n::translate_type(&c.card_type),
        desc = desc,
    )
}

fn format_monster(m: &MonsterInfo) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("[{}] {}", m.index, m.name));

    if let (Some(cur), Some(max)) = (m.current_hp, m.max_hp) {
        let mut hp_line = format!("  HP {cur}/{max}");
        if m.can_be_killed {
            hp_line.push_str("  > 可斩杀！");
        }
        lines.push(hp_line);
    }

    if let Some(ref intent) = m.intent {
        lines.push(format!("  意图：{}", i18n::translate_intent(intent)));
    }

    if let Some(dmg) = m.damage {
        let mut dmg_str = format!("  伤害：{dmg}");
        if let Some(hits) = m.hits
            && hits > 1
        {
            dmg_str.push_str(&format!("（×{hits}）"));
        }
        lines.push(dmg_str);
    } else {
        lines.push("  伤害：无".to_string());
    }

    if let Some(blk) = m.block
        && blk > 0
    {
        lines.push(format!("  格挡：{blk}"));
    }

    if !m.monster_powers.is_empty() {
        let pwr_str: Vec<String> = m
            .monster_powers
            .iter()
            .map(|p| format!("{}({})", p.name, p.amount))
            .collect();
        let mut line = format!("  能力：{}", pwr_str.join(" "));
        if m.is_scaling {
            line.push_str(" > 成长中！");
        }
        lines.push(line);
    }

    lines.join("\n")
}

fn build_monsters_section(state: &NormalizedState) -> String {
    if state.monsters.is_empty() {
        return String::new();
    }

    let mut lines = vec![format!("=== 怪物（{}只）===", state.monsters.len())];
    for m in &state.monsters {
        lines.push(format_monster(m));
        lines.push(String::new());
    }
    lines.join("\n")
}

fn build_hand_section(state: &NormalizedState, i18n_data: &I18n) -> String {
    if state.hand_cards.is_empty() {
        return String::new();
    }

    let mut lines = vec![format!(
        "=== 手牌（{}张 | 当前回合可用）===",
        state.hand_cards.len()
    )];
    for c in &state.hand_cards {
        lines.push(format!("  {}", format_card(c, i18n_data)));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn compact_pile(label: &str, cards: &[CardInfo], i18n_data: &I18n) -> String {
    if cards.is_empty() {
        return format!("{label}（0张）\n");
    }

    let mut id_counts: HashMap<&str, usize> = HashMap::new();
    for c in cards {
        *id_counts.entry(c.id.as_str()).or_insert(0) += 1;
    }

    let mut entries: Vec<String> = id_counts
        .into_iter()
        .map(|(id, count)| {
            let display = i18n_data.card(id).unwrap_or(id);
            if count == 1 {
                display.to_string()
            } else {
                format!("{display}×{count}")
            }
        })
        .collect();
    entries.sort();

    format!("{label}（{}张）\n  {}\n", cards.len(), entries.join(" "))
}

fn format_deck_section(cards: &[CardInfo], i18n_data: &I18n) -> String {
    if cards.is_empty() {
        return String::new();
    }

    let mut by_type: HashMap<&str, HashMap<&str, (usize, i64, bool)>> = HashMap::new();
    for c in cards {
        let entry = by_type
            .entry(c.card_type.as_str())
            .or_default()
            .entry(c.id.as_str())
            .or_insert((0, c.cost, c.upgraded));
        entry.0 += 1;
    }

    let type_order = ["ATTACK", "SKILL", "POWER", "CURSE", "STATUS"];
    let type_headers: HashMap<&str, &str> = HashMap::from([
        ("ATTACK", "攻击"),
        ("SKILL", "技能"),
        ("POWER", "能力"),
        ("CURSE", "诅咒"),
        ("STATUS", "状态"),
    ]);

    let mut lines: Vec<String> = Vec::new();
    lines.push("=== 卡组 ===".to_string());

    for &t in &type_order {
        if let Some(group) = by_type.get(t) {
            let mut entries: Vec<(&&str, &(usize, i64, bool))> = group.iter().collect();
            entries.sort_by_key(|(id, _)| i18n_data.card(id).unwrap_or(*id));

            let header = type_headers.get(t).unwrap_or(&t);
            let total: usize = entries.iter().map(|(_, (count, _, _))| count).sum();
            lines.push(format!("{header}（{total}张）："));
            for (id, (count, cost, upgraded)) in entries {
                let display = i18n_data.card(id).unwrap_or(*id);
                let desc = i18n_data
                    .card_desc(id)
                    .map(|raw| format!(" — {}", resolve_description(id, *upgraded, raw, i18n_data)))
                    .unwrap_or_default();
                let prefix = if *upgraded { "+" } else { "" };
                if *count == 1 {
                    lines.push(format!("  {prefix}{display}({cost}费){desc}"));
                } else {
                    lines.push(format!(
                        "  {prefix}{display}({cost}费)（共{count}张）{desc}"
                    ));
                }
            }
        }
    }

    lines.push(String::new());
    lines.join("\n")
}

fn build_combat(state: &NormalizedState, i18n_data: &I18n) -> String {
    let mut lines: Vec<String> = vec![
        "=== 当前状态 ===".to_string(),
        status_line(state),
        String::new(),
        "=== 任务 ===".to_string(),
        "这是进入战斗时的一次性建议。请给出整体打法：优先击杀目标、防守底线、药水/遗物注意点；不要逐回合假设后续抽牌。".to_string(),
        String::new(),
    ];

    if state.danger.no_block_against_hit {
        lines.push("注意：当前无格挡！".to_string());
    }
    if state.danger.wrath_stance {
        lines.push("注意：愤怒姿态下受到双倍伤害。".to_string());
    }

    // Monsters
    if !state.monsters.is_empty() {
        lines.push(build_monsters_section(state));
    }

    // Hand cards with descriptions
    if !state.hand_cards.is_empty() {
        lines.push(build_hand_section(state, i18n_data));
    }

    // Draw pile
    lines.push(compact_pile("=== 抽牌堆", &state.draw_pile, i18n_data));

    // Discard pile
    lines.push(compact_pile("=== 弃牌堆", &state.discard_pile, i18n_data));

    // Exhaust pile (only if non-empty)
    if !state.exhaust_cards.is_empty() {
        lines.push(compact_pile("=== 已消耗", &state.exhaust_cards, i18n_data));
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn build_card_reward(state: &NormalizedState, i18n_data: &I18n) -> String {
    let task = if state.is_boss_card_reward() {
        "请从 Boss 战后的奖励中选择一张牌，或推荐跳过。重点比较：下一幕卡组方向、成长、AOE、过牌、能量、格挡体系、Boss 遗物兼容性和卡组膨胀风险。"
    } else {
        "请从奖励中选择一张牌，或推荐跳过。重点比较：当前卡组缺口、费用曲线、攻防比例、遗物协同和短期生存压力。"
    };

    let mut lines: Vec<String> = vec![
        "=== 当前状态 ===".to_string(),
        status_line(state),
        String::new(),
        "=== 任务 ===".to_string(),
        task.to_string(),
        String::new(),
    ];

    if state.is_boss_card_reward() {
        lines.push(
            "注意：这是 Boss 战后的选牌。下一幕开始会回满血，不要把当前血量当成选牌依据。"
                .to_string(),
        );
        lines.push(String::new());
    }

    lines.push(format_deck_section(&state.master_cards, i18n_data));

    // Reward choices
    if !state.card_reward_choices.is_empty() {
        lines.push("=== 选牌 ===".to_string());
        for (i, c) in state.card_reward_choices.iter().enumerate() {
            let label = (b'A' + i as u8) as char;
            let display = i18n_data.card(&c.id).unwrap_or(&c.name);
            let desc = i18n_data
                .card_desc(&c.id)
                .map(|raw| {
                    format!(
                        " — {}",
                        resolve_description(&c.id, c.upgraded, raw, i18n_data)
                    )
                })
                .unwrap_or_default();
            lines.push(format!(
                "{label}. {name}({cost}费/{ctype}){desc}",
                name = display,
                cost = c.cost,
                ctype = i18n::translate_type(&c.card_type),
                desc = desc,
            ));
        }
        if state.skip_available {
            lines.push("跳过. 都不选".to_string());
        }
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn build_rest(state: &NormalizedState, i18n_data: &I18n) -> String {
    let mut lines: Vec<String> = vec![
        "=== 当前状态 ===".to_string(),
        status_line(state),
        String::new(),
        "=== 任务 ===".to_string(),
        "请在篝火选项中做决定。明确比较休息、锻造和特殊选项的收益，并说明当前血量是否允许贪长期收益。如果推荐锻造，必须写出要升级哪张牌。".to_string(),
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
    upgradeable.sort_by_key(|c| i18n_data.card(&c.id).unwrap_or(&c.name).to_string());

    if !upgradeable.is_empty() {
        let names: Vec<String> = upgradeable
            .into_iter()
            .map(|c| format_card(c, i18n_data))
            .collect();
        lines.push("=== 可锻造升级目标 ===".to_string());
        lines.extend(names);
        lines.push(String::new());
    }

    if !state.rest_options.is_empty() {
        lines.push("=== 选项 ===".to_string());
        let opts: Vec<String> = state
            .rest_options
            .iter()
            .map(|o| i18n::translate_rest_option(o).to_string())
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
            lines.push("血量极低，强烈建议休息。".to_string());
        } else if pct > 0.7 {
            lines.push("血量健康，可考虑锻造或挖遗物。".to_string());
        }
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn build_boss_relic(state: &NormalizedState, i18n_data: &I18n) -> String {
    let mut lines: Vec<String> = vec![
        "=== 当前状态 ===".to_string(),
        status_line(state),
        String::new(),
        "=== 任务 ===".to_string(),
        "请从 Boss 遗物中选择一个。重点比较能量、过牌、卡组方向、已有遗物、药水、下一幕压力和副作用。".to_string(),
        String::new(),
        format_deck_section(&state.master_cards, i18n_data),
    ];

    if !state.boss_relic_choices.is_empty() {
        lines.push("=== Boss 遗物 ===".to_string());
        for (i, relic) in state.boss_relic_choices.iter().enumerate() {
            let label = (b'A' + i as u8) as char;
            lines.push(format!("{label}. {relic}"));
        }
        lines.push(String::new());
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn build_event_choice(state: &NormalizedState, _i18n_data: &I18n) -> String {
    let mut lines: Vec<String> = vec![
        "=== 当前状态 ===".to_string(),
        status_line(state),
        String::new(),
        "=== 任务 ===".to_string(),
        "请在事件选项中做决定。比较血量、金币、卡组质量、遗物、诅咒/删牌/升级收益和长期风险；信息不足时明确说明不确定。".to_string(),
        String::new(),
    ];

    if let Some(name) = &state.event_name {
        lines.push("=== 事件 ===".to_string());
        lines.push(name.clone());
    }
    if let Some(body) = &state.event_body {
        lines.push(body.clone());
        lines.push(String::new());
    }

    if !state.event_choices.is_empty() {
        lines.push("=== 选项 ===".to_string());
        for (i, choice) in state.event_choices.iter().enumerate() {
            let label = (b'A' + i as u8) as char;
            lines.push(format!("{label}. {choice}"));
        }
        lines.push(String::new());
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn build_generic(state: &NormalizedState, i18n_data: &I18n) -> String {
    let mut lines: Vec<String> = vec![
        "=== 当前状态 ===".to_string(),
        status_line(state),
        String::new(),
        "=== 任务 ===".to_string(),
        "请基于当前状态给出一个简短、可执行的下一步建议。".to_string(),
        String::new(),
    ];

    if !state.monsters.is_empty() {
        lines.push(build_monsters_section(state));
    }

    if !state.hand_cards.is_empty() {
        let mut id_counts: HashMap<&str, usize> = HashMap::new();
        for c in &state.hand_cards {
            *id_counts.entry(c.id.as_str()).or_insert(0) += 1;
        }
        let cards: Vec<String> = id_counts
            .into_iter()
            .map(|(id, count)| {
                let display = i18n_data.card(id).unwrap_or(id);
                if count == 1 {
                    display.to_string()
                } else {
                    format!("{display}×{count}")
                }
            })
            .collect();
        lines.push(format!("手牌：{}", cards.join(" ")));
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn format_line() -> &'static str {
    "\n请按格式用中文回复（120字内，直接给结论）：\n推荐：\n理由：\n风险：\n吐槽："
}

pub fn build_prompt(state: &NormalizedState, i18n_data: &I18n) -> String {
    match state.screen_type.as_deref() {
        Some("CARD_REWARD") => build_card_reward(state, i18n_data),
        Some("BOSS_REWARD") => build_boss_relic(state, i18n_data),
        Some("REST") => build_rest(state, i18n_data),
        Some("EVENT") => build_event_choice(state, i18n_data),
        _ if !state.monsters.is_empty() => build_combat(state, i18n_data),
        _ => build_generic(state, i18n_data),
    }
}

#[cfg(test)]
#[path = "tests/prompt_tests.rs"]
mod tests;
