use crate::state::{DangerLevel, MonsterInfo, NormalizedState};

pub struct DeckAnalysis {
    pub total: usize,
    pub attack_count: usize,
    pub skill_count: usize,
    pub power_count: usize,
    pub duplicates: Vec<(String, usize)>,
    pub attack_ratio: f64,
}

impl DeckAnalysis {
    pub fn new(state: &NormalizedState) -> Self {
        let total = state.deck_names.len();

        let mut name_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for name in &state.deck_names {
            *name_counts.entry(name.as_str()).or_insert(0) += 1;
        }

        let mut duplicates: Vec<(String, usize)> = name_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(name, count)| (name.to_string(), count))
            .collect();
        duplicates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        // Estimate type counts from hand + deck card types from raw state.
        // We don't have type info in deck_names (just names), so use placeholder values.
        // The types are available in state.hand cards and we'll approximate from there.
        let attack_count = state
            .hand
            .iter()
            .filter(|c| c.card_type == "ATTACK")
            .count();
        let skill_count = state.hand.iter().filter(|c| c.card_type == "SKILL").count();
        let power_count = state.hand.iter().filter(|c| c.card_type == "POWER").count();

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

fn translate_class(class: &str) -> &str {
    match class {
        "IRONCLAD" => "铁甲战士",
        "THE_SILENT" => "猎人",
        "DEFECT" => "机器人",
        "WATCHER" => "观者",
        _ => class,
    }
}

fn translate_rest_option(opt: &str) -> &str {
    match opt {
        "rest" => "休息(回30%血)",
        "smith" => "锻造(升级)",
        "toke" => "回忆(移除一张牌)",
        "dig" => "挖遗物",
        "lift" => "举重(永久+1力量)",
        "recall" => "回忆钥匙",
        "girya" => "深蹲(力量)",
        _ => opt,
    }
}

fn header_line(state: &NormalizedState) -> String {
    let mut parts: Vec<String> = Vec::new();

    let prefix = danger_prefix(state);
    parts.push(prefix);

    if let Some(ref c) = state.character {
        parts.push(format!("角色：{}", translate_class(c)));
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

    parts.join("  ")
}

fn powers_line(state: &NormalizedState) -> Option<String> {
    if state.powers.is_empty() {
        return None;
    }
    let powers_str: Vec<String> = state
        .powers
        .iter()
        .map(|p| format!("{}({})", p.name, p.amount))
        .collect();
    Some(format!("能力：{}", powers_str.join(" ")))
}

fn hand_line(state: &NormalizedState) -> Option<String> {
    if state.hand.is_empty() {
        return None;
    }
    let cards: Vec<String> = state
        .hand
        .iter()
        .map(|c| {
            let up = if c.upgraded { "+" } else { "" };
            format!(
                "{up}{}({}费/{})",
                c.name,
                c.cost,
                translate_type(&c.card_type)
            )
        })
        .collect();
    Some(format!("手牌：{}", cards.join(" ")))
}

fn translate_type(t: &str) -> &str {
    match t {
        "ATTACK" => "攻击",
        "SKILL" => "技能",
        "POWER" => "能力",
        "CURSE" => "诅咒",
        "STATUS" => "状态",
        _ => t,
    }
}

fn monsters_line(state: &NormalizedState) -> Option<String> {
    if state.monsters.is_empty() {
        return None;
    }
    let strs: Vec<String> = state.monsters.iter().map(format_monster).collect();
    Some(format!("怪物：{}", strs.join(" | ")))
}

fn format_monster(m: &MonsterInfo) -> String {
    let mut s = m.name.clone();
    if let (Some(cur), Some(max)) = (m.current_hp, m.max_hp) {
        s.push_str(&format!("({cur}/{max})"));
    }
    if let Some(ref intent) = m.intent {
        s.push_str(&format!("[{}]", translate_intent(intent)));
    }
    if let Some(dmg) = m.damage {
        s.push_str(&format!("[{dmg}伤]"));
    }
    s
}

fn translate_intent(intent: &str) -> &str {
    match intent {
        "ATTACK" => "攻击",
        "ATTACK_BUFF" => "攻击+增益",
        "ATTACK_DEBUFF" => "攻击+减益",
        "ATTACK_DEFEND" => "攻击+防御",
        "BUFF" => "增益",
        "DEBUFF" => "减益",
        "STRONG_DEBUFF" => "强力减益",
        "DEBUG" => "调试",
        "DEFEND" => "防御",
        "DEFEND_DEBUFF" => "防御+减益",
        "DEFEND_BUFF" => "防御+增益",
        "ESCAPE" => "逃跑",
        "MAGIC" => "法术",
        "NONE" => "无",
        "SLEEP" => "睡眠",
        "STUN" => "眩晕",
        "UNKNOWN" => "未知",
        _ => intent,
    }
}

fn deck_summary(state: &NormalizedState) -> Option<String> {
    if state.deck_names.is_empty() {
        return None;
    }

    let deck = DeckAnalysis::new(state);
    let mut parts: Vec<String> = Vec::new();

    let mut type_parts: Vec<String> = Vec::new();
    if deck.attack_count > 0 {
        type_parts.push(format!("攻击{}", deck.attack_count));
    }
    if deck.skill_count > 0 {
        type_parts.push(format!("技能{}", deck.skill_count));
    }
    if deck.power_count > 0 {
        type_parts.push(format!("能力{}", deck.power_count));
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

    Some(format!("卡组({}张)：{}", deck.total, parts.join("   ")))
}

fn format_line() -> &'static str {
    "请按格式回复（120字内）：\n推荐：\n理由：\n风险：\n吐槽："
}

// ── Templates ──

fn build_combat(state: &NormalizedState) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("你是《杀戮尖塔》中文直播助手。根据当前战斗状态给出具体出牌建议。".to_string());

    lines.push(header_line(state));

    if let Some(p) = powers_line(state) {
        lines.push(p);
    }
    if let Some(h) = hand_line(state) {
        lines.push(h);
    }
    if let Some(m) = monsters_line(state) {
        lines.push(m);
    }
    if let Some(d) = deck_summary(state) {
        lines.push(d);
    }

    // Contextual hints
    if state.danger.no_block_against_hit {
        lines.push("注意：当前无格挡！".to_string());
    }
    if state.danger.wrath_stance {
        lines.push("注意：愤怒姿态下受到双倍伤害。".to_string());
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn build_card_reward(state: &NormalizedState) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("你是《杀戮尖塔》中文直播助手。根据卡组和选牌给出建议。".to_string());

    // Minimal header (no combat data)
    let mut parts: Vec<String> = Vec::new();
    parts.push(danger_prefix(state));
    if let Some(ref c) = state.character {
        parts.push(format!("角色：{}", translate_class(c)));
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
    lines.push(parts.join("  "));

    if let Some(d) = deck_summary(state) {
        lines.push(d);
    }

    if !state.card_reward_choices.is_empty() {
        let choices: Vec<String> = state
            .card_reward_choices
            .iter()
            .enumerate()
            .map(|(i, name)| format!("  {}. {name}", (b'A' + i as u8) as char))
            .collect();
        lines.push(format!("选牌：\n{}", choices.join("\n")));
    }

    // Deck composition warning
    let deck = DeckAnalysis::new(state);
    if deck.total > 0 && deck.attack_ratio > 0.5 {
        lines.push("提示：攻击牌占比偏高，建议补充技能牌或能力牌。".to_string());
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

fn build_rest(state: &NormalizedState) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("你是《杀戮尖塔》中文直播助手。根据休息处选项给出建议。".to_string());

    lines.push(header_line(state));

    if !state.rest_options.is_empty() {
        let opts: Vec<String> = state
            .rest_options
            .iter()
            .map(|o| translate_rest_option(o).to_string())
            .collect();
        lines.push(format!("选项：{}", opts.join("  ")));
    }

    // HP hint
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

    lines.push("你是《杀戮尖塔》中文直播助手。根据当前游戏状态给出简单建议。".to_string());

    lines.push(header_line(state));

    if let Some(p) = powers_line(state) {
        lines.push(p);
    }
    if let Some(h) = hand_line(state) {
        lines.push(h);
    }
    if let Some(m) = monsters_line(state) {
        lines.push(m);
    }
    if let Some(d) = deck_summary(state) {
        lines.push(d);
    }
    if !state.relics.is_empty() {
        lines.push(format!("遗物：{}", state.relics.join(", ")));
    }
    if !state.potions.is_empty() {
        lines.push(format!("药水：{}", state.potions.join(", ")));
    }

    lines.push(format_line().to_string());
    lines.join("\n")
}

/// Dispatch to the appropriate template based on screen type.
pub fn build_prompt(state: &NormalizedState) -> String {
    match state.screen_type.as_deref() {
        Some("CARD_REWARD") => build_card_reward(state),
        Some("REST") | Some("CHEST") if !state.rest_options.is_empty() => build_rest(state),
        _ if !state.monsters.is_empty() => build_combat(state),
        _ => build_generic(state),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{DangerFlags, DangerLevel, MonsterInfo};

    fn test_state() -> NormalizedState {
        NormalizedState {
            screen_type: Some("NONE".to_string()),
            room_type: Some("MonsterRoom".to_string()),
            character: Some("IRONCLAD".to_string()),
            floor: Some(1),
            current_hp: Some(68),
            max_hp: Some(75),
            gold: Some(99),
            energy: Some(3),
            block: Some(6),
            powers: vec![],
            hand: vec![],
            monsters: vec![],
            card_reward_choices: vec![],
            relics: vec![],
            potions: vec![],
            deck_names: vec![],
            incoming_damage: 0,
            rest_options: vec![],
            danger: DangerFlags {
                hp_critical: false,
                incoming_lethal: false,
                no_block_against_hit: false,
                any_monster_attacking: false,
                wrath_stance: false,
                level: DangerLevel::Safe,
            },
        }
    }

    #[test]
    fn combat_prompt_has_monster_intent() {
        let state = NormalizedState {
            monsters: vec![MonsterInfo {
                name: "Jaw Worm".into(),
                current_hp: Some(44),
                max_hp: Some(46),
                intent: Some("ATTACK".into()),
                damage: Some(12),
            }],
            danger: DangerFlags {
                any_monster_attacking: true,
                level: DangerLevel::Caution,
                ..test_state().danger
            },
            ..test_state()
        };

        let prompt = build_prompt(&state);
        assert!(prompt.contains("Jaw Worm"));
        assert!(prompt.contains("12伤"));
    }

    #[test]
    fn card_reward_prompt_lists_choices() {
        let state = NormalizedState {
            screen_type: Some("CARD_REWARD".into()),
            character: Some("IRONCLAD".into()),
            floor: Some(3),
            current_hp: Some(62),
            max_hp: Some(75),
            card_reward_choices: vec!["Uppercut".into(), "Anger".into(), "Headbutt".into()],
            deck_names: vec!["Strike".into(), "Defend".into()],
            ..test_state()
        };

        let prompt = build_prompt(&state);
        assert!(prompt.contains("Uppercut"));
        assert!(prompt.contains("Anger"));
        assert!(prompt.contains("Headbutt"));
        assert!(prompt.contains("卡组(2张)"));
    }

    #[test]
    fn card_reward_prompt_mentions_deck() {
        let state = NormalizedState {
            screen_type: Some("CARD_REWARD".into()),
            deck_names: vec!["Strike".into(), "Strike".into(), "Defend".into()],
            ..test_state()
        };

        let prompt = build_prompt(&state);
        assert!(prompt.contains("卡组"));
        assert!(prompt.contains("Strike"));
    }

    #[test]
    fn prompt_format_has_required_fields() {
        let prompt = build_prompt(&test_state());
        assert!(prompt.contains("推荐："));
        assert!(prompt.contains("理由："));
        assert!(prompt.contains("风险："));
        assert!(prompt.contains("吐槽："));
    }

    #[test]
    fn rest_prompt_has_translated_options() {
        let state = NormalizedState {
            screen_type: Some("REST".into()),
            character: Some("IRONCLAD".into()),
            current_hp: Some(25),
            max_hp: Some(75),
            floor: Some(5),
            rest_options: vec!["rest".into(), "smith".into()],
            danger: DangerFlags {
                hp_critical: true,
                level: DangerLevel::Danger,
                ..test_state().danger
            },
            ..test_state()
        };

        let prompt = build_prompt(&state);
        assert!(prompt.contains("休息"));
        assert!(prompt.contains("锻造"));
    }

    #[test]
    fn danger_tone_shows_in_prompt() {
        let combat = NormalizedState {
            monsters: vec![MonsterInfo {
                name: "Enemy".into(),
                current_hp: Some(40),
                max_hp: Some(40),
                intent: Some("ATTACK".into()),
                damage: Some(30),
            }],
            current_hp: Some(20),
            max_hp: Some(80),
            incoming_damage: 30,
            danger: DangerFlags {
                incoming_lethal: true,
                any_monster_attacking: true,
                level: DangerLevel::Danger,
                ..test_state().danger
            },
            ..test_state()
        };

        let prompt = build_prompt(&combat);
        assert!(prompt.contains("危险"));

        let safe = NormalizedState {
            current_hp: Some(72),
            max_hp: Some(75),
            danger: DangerFlags {
                level: DangerLevel::Safe,
                ..test_state().danger
            },
            ..test_state()
        };
        let prompt2 = build_prompt(&safe);
        assert!(prompt2.contains("形势不错"));
    }
}
