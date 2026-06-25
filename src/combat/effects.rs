use crate::locales::EffectParserLocale;
use crate::state::CardInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetType {
    Targeted,
    AoE,
    RandomTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageEffect {
    pub amount: i16,
    pub hits: i16,
    pub target_type: TargetType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExhaustKind {
    None,
    Self_,
    All,
    NonAttacks,
    Attacks,
    Random,
    Chosen,
    FiendFire,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardEffect {
    pub damage: Option<DamageEffect>,
    pub energy_gain: i16,
    pub strength_gain: i16,
    pub vulnerable: Option<i16>,
    pub stance: StanceEffect,
    pub mantra_gain: i16,
    pub execute: Option<i16>,
    pub exhaust: ExhaustKind,
    pub x_cost: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StanceEffect {
    None,
    EnterWrath,
    EnterCalm,
    ExitStance,
    EnterDivinity,
}

fn extract_first_integer(s: &str) -> Option<i16> {
    let digits: String = s
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

fn parse_hits_suffix(segment: &str) -> Option<i16> {
    if segment.contains("两次") || segment.contains("2 times") || segment.contains("twice") {
        return Some(2);
    }

    if segment.contains("X次") || segment.contains("x次") || segment.contains("X times") {
        return Some(0);
    }

    if !segment.contains('次') && !segment.to_lowercase().contains("times") {
        return None;
    }

    for needle in ["次", "times"] {
        if let Some(pos) = segment.to_lowercase().rfind(needle) {
            let before = &segment[..pos];
            let mut digits = String::new();
            for c in before.chars().rev() {
                if c.is_ascii_digit() {
                    digits.insert(0, c);
                } else if !digits.is_empty() {
                    break;
                }
            }
            if let Ok(n) = digits.parse::<i16>()
                && n > 0
            {
                return Some(n);
            }
        }
    }

    None
}

fn parse_damage_multiplier_hits(desc: &str) -> Option<(i16, i16)> {
    for segment in desc.split('。') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let Some(dmg) = extract_first_integer(segment) else {
            continue;
        };
        if dmg <= 0 || dmg > 1000 {
            continue;
        }

        if let Some(hits) = parse_hits_suffix(segment) {
            return Some((dmg, hits));
        }

        return Some((dmg, 1));
    }
    for segment in desc.split('.') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let Some(dmg) = extract_first_integer(segment) else {
            continue;
        };
        if dmg <= 0 || dmg > 1000 {
            continue;
        }
        if let Some(hits) = parse_hits_suffix(segment) {
            return Some((dmg, hits));
        }
        return Some((dmg, 1));
    }
    None
}

fn parse_target_type(desc: &str, locale: &EffectParserLocale) -> TargetType {
    let lower = desc.to_lowercase();
    if locale
        .aoe_keywords
        .iter()
        .any(|k| lower.contains(&k.to_lowercase()))
    {
        TargetType::AoE
    } else if locale
        .random_keywords
        .iter()
        .any(|k| lower.contains(&k.to_lowercase()))
    {
        TargetType::RandomTarget
    } else {
        TargetType::Targeted
    }
}

fn parse_damage(desc: &str, card_type: &str, locale: &EffectParserLocale) -> Option<DamageEffect> {
    if card_type != "ATTACK" {
        return None;
    }
    let (amount, hits) = parse_damage_multiplier_hits(desc)?;
    let target_type = parse_target_type(desc, locale);
    let effective_hits = if hits == 0 { 1 } else { hits };
    Some(DamageEffect {
        amount,
        hits: effective_hits,
        target_type,
    })
}

fn parse_energy_gain(desc: &str) -> i16 {
    desc.matches("[E]").count() as i16 + desc.matches("能量").count() as i16
}

fn parse_strength_gain(desc: &str, locale: &EffectParserLocale) -> i16 {
    let loss = &locale.strength_lose_keyword;
    for segment in desc.split('。') {
        let segment = segment.trim();
        if segment.contains(&locale.strength_gain_keyword) || segment.contains("Strength") {
            if segment.contains(loss) {
                continue;
            }
            if let Some(n) = extract_first_integer(segment) {
                return n;
            }
        }
    }
    for segment in desc.split('.') {
        let segment = segment.trim();
        if segment.to_lowercase().contains("strength")
            || segment.contains(&locale.strength_gain_keyword)
        {
            if segment.to_lowercase().contains(&loss.to_lowercase()) {
                continue;
            }
            if let Some(n) = extract_first_integer(segment) {
                return n;
            }
        }
    }
    0
}

fn parse_vulnerable(desc: &str, locale: &EffectParserLocale) -> Option<i16> {
    let keyword = &locale.vulnerable_keyword;
    for segment in desc.split('。') {
        let segment = segment.trim();
        if segment.contains(keyword) || segment.contains("Vulnerable") {
            let n = extract_first_integer(segment).unwrap_or(1);
            return Some(n);
        }
    }
    for segment in desc.split('.') {
        let segment = segment.trim();
        if segment.to_lowercase().contains("vulnerable") || segment.contains(keyword) {
            let n = extract_first_integer(segment).unwrap_or(1);
            return Some(n);
        }
    }
    None
}

fn parse_stance(desc: &str, locale: &EffectParserLocale) -> StanceEffect {
    let lower = desc.to_lowercase();

    let wrath_match = locale
        .enter_wrath_keywords
        .iter()
        .all(|k| lower.contains(&k.to_lowercase()));
    let calm_match = locale
        .enter_calm_keywords
        .iter()
        .all(|k| lower.contains(&k.to_lowercase()));
    let exit_match = locale
        .exit_stance_keywords
        .iter()
        .all(|k| lower.contains(&k.to_lowercase()));
    let divinity_match = locale
        .enter_divinity_keywords
        .iter()
        .all(|k| lower.contains(&k.to_lowercase()));

    if wrath_match {
        StanceEffect::EnterWrath
    } else if calm_match {
        StanceEffect::EnterCalm
    } else if exit_match {
        StanceEffect::ExitStance
    } else if divinity_match {
        StanceEffect::EnterDivinity
    } else {
        StanceEffect::None
    }
}

fn parse_mantra(desc: &str, locale: &EffectParserLocale) -> i16 {
    let keyword = &locale.mantra_keyword;
    if desc.contains(keyword) || desc.to_lowercase().contains("mantra") {
        extract_first_integer(desc).unwrap_or(1)
    } else {
        0
    }
}

fn parse_execute(desc: &str, locale: &EffectParserLocale) -> Option<i16> {
    let lower = desc.to_lowercase();
    if locale
        .execute_keywords
        .iter()
        .any(|k| lower.contains(&k.to_lowercase()))
    {
        return extract_first_integer(desc);
    }
    if lower.contains("hp is") || lower.contains("hp or less") || lower.contains("set its hp to 0")
    {
        return extract_first_integer(desc);
    }
    None
}

fn parse_exhaust(desc: &str, locale: &EffectParserLocale) -> ExhaustKind {
    let kw = &locale.exhaust_keyword;
    let lower = desc.to_lowercase();
    let has_exhaust = lower.contains(&kw.to_lowercase()) || lower.contains("exhaust");
    if !has_exhaust {
        return ExhaustKind::None;
    }
    if lower.contains("所有非攻击牌") || lower.contains("all non-attack") {
        ExhaustKind::NonAttacks
    } else if lower.contains("所有攻击牌") || lower.contains("all attack") {
        ExhaustKind::Attacks
    } else if lower.contains("所有手牌") || lower.contains("所有牌") || lower.contains("all cards")
    {
        ExhaustKind::All
    } else if lower.contains("随机") || lower.contains("random") {
        ExhaustKind::Random
    } else if lower.contains("选择")
        || lower.contains("任意")
        || lower.contains("choose")
        || (lower.contains("一张牌") && lower.contains(&kw.to_lowercase()))
    {
        ExhaustKind::Chosen
    } else {
        ExhaustKind::Self_
    }
}

pub fn parse_card_effect(card: &CardInfo, locale: &EffectParserLocale) -> Option<CardEffect> {
    let desc = &card.description;

    let damage = parse_damage(desc, &card.card_type, locale);

    let effect = CardEffect {
        damage,
        energy_gain: parse_energy_gain(desc),
        strength_gain: parse_strength_gain(desc, locale),
        vulnerable: parse_vulnerable(desc, locale),
        stance: parse_stance(desc, locale),
        mantra_gain: parse_mantra(desc, locale),
        execute: parse_execute(desc, locale),
        exhaust: parse_exhaust(desc, locale),
        x_cost: desc.contains('X')
            || desc.to_lowercase().contains("x次")
            || desc.to_lowercase().contains("x times")
            || desc.contains("花费所有")
            || desc.to_lowercase().contains("spend all"),
    };

    if is_scan_relevant(&effect) {
        Some(effect)
    } else {
        None
    }
}

pub fn is_scan_relevant(effect: &CardEffect) -> bool {
    effect.damage.is_some()
        || effect.energy_gain > 0
        || effect.strength_gain > 0
        || effect.vulnerable.is_some()
        || effect.stance != StanceEffect::None
        || effect.mantra_gain > 0
        || effect.execute.is_some()
        || effect.exhaust != ExhaustKind::None
        || effect.x_cost
}

#[cfg(test)]
mod tests {
    use crate::locales::EffectParserLocale;

    use super::*;
    use crate::state::CardInfo;

    fn zh_locale() -> EffectParserLocale {
        EffectParserLocale {
            damage_keyword: "造成".into(),
            aoe_keywords: vec!["所有敌人".into(), "全部敌人".into(), "全体敌人".into()],
            random_keywords: vec!["随机".into()],
            vulnerable_keyword: "易伤".into(),
            strength_gain_keyword: "力量".into(),
            strength_lose_keyword: "失去".into(),
            enter_wrath_keywords: vec!["进入".into(), "愤怒".into()],
            enter_calm_keywords: vec!["进入".into(), "宁静".into()],
            exit_stance_keywords: vec!["退出".into(), "姿态".into()],
            enter_divinity_keywords: vec!["进入".into(), "神格".into()],
            mantra_keyword: "真言".into(),
            execute_keywords: vec!["生命值".into(), "小于等于".into()],
            exhaust_keyword: "消耗".into(),
        }
    }

    fn card(desc: &str) -> CardInfo {
        CardInfo {
            id: "TestCard".into(),
            name: "Test".into(),
            cost: 1,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: Some("test-uuid".into()),
            description: desc.into(),
            price: None,
            playable: true,
            has_target: true,
        }
    }

    fn skill(desc: &str) -> CardInfo {
        CardInfo {
            id: "TestSkill".into(),
            name: "Test".into(),
            cost: 1,
            card_type: "SKILL".into(),
            upgraded: false,
            uuid: Some("test-uuid".into()),
            description: desc.into(),
            price: None,
            playable: true,
            has_target: false,
        }
    }

    #[test]
    fn parse_strike_damage() {
        let c = card("造成 6 点伤害。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(
            e.damage,
            Some(DamageEffect {
                amount: 6,
                hits: 1,
                target_type: TargetType::Targeted,
            })
        );
    }

    #[test]
    fn parse_multi_hit_damage() {
        let c = card("造成 2 点伤害 5 次。 消耗 。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(
            e.damage,
            Some(DamageEffect {
                amount: 2,
                hits: 5,
                target_type: TargetType::Targeted,
            })
        );
    }

    #[test]
    fn parse_two_hit_damage() {
        let c = card("造成 7 点伤害两次。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.damage.unwrap().hits, 2);
    }

    #[test]
    fn parse_aoe_damage() {
        let c = card("对所有敌人造成 4 点伤害。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.damage.unwrap().target_type, TargetType::AoE);
    }

    #[test]
    fn parse_random_target() {
        let c = card("随机对敌人造成 3 点伤害 4 次。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.damage.unwrap().target_type, TargetType::RandomTarget);
    }

    #[test]
    fn parse_x_cost_damage() {
        let c = card("对所有敌人造成 8 点伤害X次。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert!(e.x_cost);
        assert_eq!(e.damage.unwrap().target_type, TargetType::AoE);
    }

    #[test]
    fn parse_vulnerable() {
        let c = card("造成 10 点伤害。 给予 3 层 易伤 。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.vulnerable, Some(3));
    }

    #[test]
    fn parse_energy_gain() {
        let c = skill("获得 [E] [E] [E] 。 失去 3 点生命。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.energy_gain, 3);
    }

    #[test]
    fn parse_strength_gain() {
        let c = skill("获得 4 点 力量 。 你的回合结束时，失去 4 点 力量 。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.strength_gain, 4);
    }

    #[test]
    fn parse_exhaust_self() {
        let c = skill("获得 [E] [E] 。 消耗 。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.exhaust, ExhaustKind::Self_);
    }

    #[test]
    fn parse_exhaust_all() {
        let c = skill("消耗 所有手牌。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.exhaust, ExhaustKind::All);
    }

    #[test]
    fn parse_execute() {
        let c = skill("如果目标生命值小于等于 30 ，将其生命值变为0。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.execute, Some(30));
    }

    #[test]
    fn parse_stance_enter_wrath() {
        let c = skill("进入 愤怒 。 造成 6 点伤害。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.stance, StanceEffect::EnterWrath);
    }

    #[test]
    fn parse_stance_enter_calm() {
        let c = skill("进入 宁静 。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.stance, StanceEffect::EnterCalm);
    }

    #[test]
    fn parse_stance_exit() {
        let c = skill("退出 姿态 。 获得 4 点 力量 。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.stance, StanceEffect::ExitStance);
    }

    #[test]
    fn parse_stance_enter_divinity() {
        let c = skill("进入 神格 。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.stance, StanceEffect::EnterDivinity);
    }

    #[test]
    fn parse_mantra() {
        let c = skill("获得 3 层 真言 。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.mantra_gain, 3);
    }

    #[test]
    fn pure_block_card_rejected() {
        let c = skill("获得 5 点 格挡 。");
        assert!(parse_card_effect(&c, &zh_locale()).is_none());
    }

    #[test]
    fn pure_dex_card_rejected() {
        let c = skill("获得 2 点 敏捷 。");
        assert!(parse_card_effect(&c, &zh_locale()).is_none());
    }

    #[test]
    fn strength_double_not_counted_as_strength_gain() {
        let c = skill("将你的 力量 翻倍。");
        let e = parse_card_effect(&c, &zh_locale());
        assert!(e.is_none() || e.unwrap().strength_gain == 0);
    }
}
