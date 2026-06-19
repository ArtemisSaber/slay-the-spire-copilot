use crate::i18n::{self, I18n};
use crate::state::{CardInfo, DangerLevel, MonsterInfo, NormalizedState};
use std::collections::HashMap;

pub struct DeckAnalysis {
    pub total: usize,
    pub attack_count: usize,
    pub skill_count: usize,
    pub power_count: usize,
    pub duplicates: Vec<(String, usize)>,
    pub attack_ratio: f64,
}

impl DeckAnalysis {
    pub fn from_cards(cards: &[CardInfo]) -> Self {
        let total = cards.len();

        let attack_count = cards.iter().filter(|c| c.card_type == "ATTACK").count();
        let skill_count = cards.iter().filter(|c| c.card_type == "SKILL").count();
        let power_count = cards.iter().filter(|c| c.card_type == "POWER").count();

        let mut name_counts: HashMap<&str, usize> = HashMap::new();
        for c in cards {
            *name_counts.entry(c.name.as_str()).or_insert(0) += 1;
        }

        let mut duplicates: Vec<(String, usize)> = name_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(name, count)| (name.to_string(), count))
            .collect();
        duplicates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let attack_ratio = if total > 0 {
            attack_count as f64 / total as f64
        } else {
            0.0
        };

        DeckAnalysis {
            total,
            attack_count,
            skill_count,
            power_count,
            duplicates,
            attack_ratio,
        }
    }
}

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

fn format_card(c: &CardInfo, i18n_data: &I18n) -> String {
    let rarity = match c.card_type.as_str() {
        "ATTACK" | "SKILL" | "POWER" => "",
        _ => "",
    };
    let desc = i18n_data
        .card_desc(&c.name)
        .or_else(|| {
            // Try by id-like lookup
            c.name
                .chars()
                .filter(|ch| ch.is_alphanumeric() || *ch == '_' || *ch == ' ')
                .collect::<String>()
                .split_whitespace()
                .next()
                .and_then(|id| i18n_data.card_desc(id))
        })
        .map(|d| format!(" — {d}"))
        .unwrap_or_default();
    let _ = rarity;
    format!(
        "{up}{name}({cost}费/{ctype}){desc}",
        up = if c.upgraded { "+" } else { "" },
        name = c.name,
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

fn compact_pile(name: &str, cards: &[CardInfo]) -> String {
    if cards.is_empty() {
        return format!("{name}（0张）\n");
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

    format!("{name}（{}张）\n  {}\n", cards.len(), entries.join(" "))
}

fn build_combat(state: &NormalizedState, i18n_data: &I18n) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("=== 当前状态 ===".to_string());
    lines.push(status_line(state));
    lines.push(String::new());

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
    lines.push(compact_pile("=== 抽牌堆", &state.draw_pile));

    // Discard pile
    lines.push(compact_pile("=== 弃牌堆", &state.discard_pile));

    // Exhaust pile (only if non-empty)
    if !state.exhaust_cards.is_empty() {
        lines.push(compact_pile("=== 已消耗", &state.exhaust_cards));
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn build_card_reward(state: &NormalizedState, i18n_data: &I18n) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("=== 当前状态 ===".to_string());
    lines.push(status_line(state));
    lines.push(String::new());

    // Master deck
    if !state.master_cards.is_empty() {
        let deck = DeckAnalysis::from_cards(&state.master_cards);
        let mut parts: Vec<String> = Vec::new();

        parts.push(format!("完整卡组（{}张）", deck.total));

        let mut type_parts: Vec<String> = Vec::new();
        if deck.attack_count > 0 {
            type_parts.push(format!("攻击({})", deck.attack_count));
        }
        if deck.skill_count > 0 {
            type_parts.push(format!("技能({})", deck.skill_count));
        }
        if deck.power_count > 0 {
            type_parts.push(format!("能力({})", deck.power_count));
        }
        if !type_parts.is_empty() {
            parts.push(type_parts.join(" "));
        }

        if !deck.duplicates.is_empty() {
            let dup_str: Vec<String> = deck
                .duplicates
                .iter()
                .map(|(name, count)| format!("{name}×{count}"))
                .collect();
            parts.push(format!("重复：{}", dup_str.join(" ")));
        }

        lines.push(format!("=== {}", parts.join("  ")));
        lines.push(String::new());
    }

    // Reward choices
    if !state.card_reward_choices.is_empty() {
        lines.push("=== 选牌 ===".to_string());
        for (i, c) in state.card_reward_choices.iter().enumerate() {
            let label = (b'A' + i as u8) as char;
            let desc = i18n_data
                .card_desc(&c.name)
                .map(|d| format!(" — {d}"))
                .unwrap_or_default();
            lines.push(format!(
                "{label}. {name}({cost}费/{ctype}/{rarity}){desc}",
                name = c.name,
                cost = c.cost,
                ctype = i18n::translate_type(&c.card_type),
                rarity = "普通", // rarity not extracted yet
                desc = desc,
            ));
        }

        let deck = DeckAnalysis::from_cards(&state.master_cards);
        if deck.total > 0 && deck.attack_ratio > 0.5 {
            lines.push("提示：攻击牌占{}%，建议补充技能牌或能力牌。".to_string());
        }
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn build_rest(state: &NormalizedState, _i18n_data: &I18n) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("=== 当前状态 ===".to_string());
    lines.push(status_line(state));
    lines.push(String::new());

    // Upgradeable cards (unupgraded cards in master deck)
    let upgradeable: Vec<&str> = state
        .master_cards
        .iter()
        .filter(|c| !c.upgraded && c.card_type != "CURSE" && c.card_type != "STATUS")
        .map(|c| c.name.as_str())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .take(10)
        .collect();
    let mut upgradeable: Vec<&&str> = upgradeable.iter().collect();
    upgradeable.sort();

    if !upgradeable.is_empty() {
        let names: Vec<&str> = upgradeable.into_iter().copied().collect();
        lines.push("=== 可升级卡牌 ===".to_string());
        lines.push(names.join(" "));
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

fn build_generic(state: &NormalizedState) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("=== 当前状态 ===".to_string());
    lines.push(status_line(state));
    lines.push(String::new());

    if !state.monsters.is_empty() {
        lines.push(build_monsters_section(state));
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
        lines.push(format!("手牌：{}", cards.join(" ")));
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn format_line() -> &'static str {
    "\n请按格式用中文回复（120字内）：\n推荐：\n理由：\n风险：\n吐槽："
}

pub fn build_prompt(state: &NormalizedState, i18n_data: &I18n) -> String {
    match state.screen_type.as_deref() {
        Some("CARD_REWARD") => build_card_reward(state, i18n_data),
        Some("REST") => build_rest(state, i18n_data),
        _ if !state.monsters.is_empty() => build_combat(state, i18n_data),
        _ => build_generic(state),
    }
}

#[cfg(test)]
#[path = "tests/prompt_tests.rs"]
mod tests;
