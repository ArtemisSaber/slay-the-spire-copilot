use crate::locales::EffectParserLocale;
use crate::state::CardInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetType {
    Targeted,
    AoE,
    RandomTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitCount {
    Fixed(i16),
    XTimes,
    XPlus(i16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageEffect {
    pub amount: i16,
    pub hits: HitCount,
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

fn parse_hits_suffix(segment: &str) -> Option<HitCount> {
    if segment.contains("两次") || segment.contains("2 times") || segment.contains("twice") {
        return Some(HitCount::Fixed(2));
    }

    let lower = segment.to_lowercase();

    if lower.contains("x+") || lower.contains("x +") {
        let after_x = lower.split('x').next_back().unwrap_or("");
        let plus_digits: String = after_x
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(offset) = plus_digits.parse::<i16>() {
            return Some(HitCount::XPlus(offset));
        }
    }

    let cn_xplus = segment.contains("X+") || segment.contains("x+");
    if cn_xplus
        && let Some(s) = segment
            .split(|c: char| !c.is_ascii_digit())
            .find(|s| !s.is_empty())
        && let Ok(offset) = s.parse::<i16>()
    {
        return Some(HitCount::XPlus(offset));
    }

    if segment.contains("X次")
        || segment.contains("x次")
        || lower.contains("x 次")
        || lower.contains("X times")
        || lower.contains("x times")
    {
        return Some(HitCount::XTimes);
    }

    if !segment.contains('次') && !lower.contains("times") {
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
                return Some(HitCount::Fixed(n));
            }
        }
    }

    None
}

fn parse_damage_multiplier_hits(desc: &str) -> Option<(i16, HitCount)> {
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

        return Some((dmg, HitCount::Fixed(1)));
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
        return Some((dmg, HitCount::Fixed(1)));
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
    Some(DamageEffect {
        amount,
        hits,
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
                hits: HitCount::Fixed(1),
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
                hits: HitCount::Fixed(5),
                target_type: TargetType::Targeted,
            })
        );
    }

    #[test]
    fn parse_two_hit_damage() {
        let c = card("造成 7 点伤害两次。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert_eq!(e.damage.unwrap().hits, HitCount::Fixed(2));
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

    #[test]
    fn parse_x_plus_one() {
        let c = card("造成 7 点伤害 X+1 次。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert!(e.x_cost);
        assert_eq!(
            e.damage,
            Some(DamageEffect {
                amount: 7,
                hits: HitCount::XPlus(1),
                target_type: TargetType::Targeted,
            })
        );
    }

    #[test]
    fn parse_x_cost_hits() {
        let c = card("造成 8 点伤害 X 次。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert!(e.x_cost);
        assert_eq!(e.damage.unwrap().hits, HitCount::XTimes);
    }

    fn en_locale() -> EffectParserLocale {
        EffectParserLocale {
            damage_keyword: "deal".into(),
            aoe_keywords: vec!["all enemies".into(), "ALL enemies".into()],
            random_keywords: vec!["random".into()],
            vulnerable_keyword: "Vulnerable".into(),
            strength_gain_keyword: "Strength".into(),
            strength_lose_keyword: "lose".into(),
            enter_wrath_keywords: vec!["Enter".into(), "Wrath".into()],
            enter_calm_keywords: vec!["Enter".into(), "Calm".into()],
            exit_stance_keywords: vec!["Exit".into(), "stance".into()],
            enter_divinity_keywords: vec!["Enter".into(), "Divinity".into()],
            mantra_keyword: "Mantra".into(),
            execute_keywords: vec![
                "HP is".into(),
                "set its HP to 0".into(),
                "HP or less".into(),
            ],
            exhaust_keyword: "Exhaust".into(),
        }
    }

    fn ja_locale() -> EffectParserLocale {
        EffectParserLocale {
            damage_keyword: "与える".into(),
            aoe_keywords: vec!["すべての敵".into(), "全ての敵".into()],
            random_keywords: vec!["ランダム".into()],
            vulnerable_keyword: "脆弱".into(),
            strength_gain_keyword: "筋力".into(),
            strength_lose_keyword: "失う".into(),
            enter_wrath_keywords: vec!["憤怒".into(), "入る".into()],
            enter_calm_keywords: vec!["平静".into(), "入る".into()],
            exit_stance_keywords: vec!["構え".into(), "解除".into()],
            enter_divinity_keywords: vec!["神格".into(), "入る".into()],
            mantra_keyword: "マントラ".into(),
            execute_keywords: vec!["HPが".into()],
            exhaust_keyword: "廃棄".into(),
        }
    }

    fn ko_locale() -> EffectParserLocale {
        EffectParserLocale {
            damage_keyword: "줍니다".into(),
            aoe_keywords: vec!["모든 적".into(), "모든 적에게".into()],
            random_keywords: vec!["무작위".into()],
            vulnerable_keyword: "취약".into(),
            strength_gain_keyword: "힘".into(),
            strength_lose_keyword: "잃습니다".into(),
            enter_wrath_keywords: vec!["분노".into(), "들어갑니다".into()],
            enter_calm_keywords: vec!["평온".into(), "들어갑니다".into()],
            exit_stance_keywords: vec!["자세".into(), "해제".into()],
            enter_divinity_keywords: vec!["신격".into(), "들어갑니다".into()],
            mantra_keyword: "만트라".into(),
            execute_keywords: vec!["HP가".into()],
            exhaust_keyword: "소멸".into(),
        }
    }

    fn power(desc: &str) -> CardInfo {
        CardInfo {
            id: "TestPower".into(),
            name: "Test".into(),
            cost: 1,
            card_type: "POWER".into(),
            upgraded: false,
            uuid: Some("test-uuid".into()),
            description: desc.into(),
            price: None,
            playable: true,
            has_target: false,
        }
    }

    #[test]
    fn extract_first_integer_no_digits() {
        assert_eq!(extract_first_integer("no numbers here"), None);
    }

    #[test]
    fn extract_first_integer_with_digits() {
        assert_eq!(extract_first_integer("deal 42 damage"), Some(42));
    }

    #[test]
    fn parse_hits_suffix_en_2_times() {
        assert_eq!(parse_hits_suffix("2 times"), Some(HitCount::Fixed(2)));
    }

    #[test]
    fn parse_hits_suffix_en_twice() {
        assert_eq!(
            parse_hits_suffix("deal damage twice"),
            Some(HitCount::Fixed(2))
        );
    }

    #[test]
    fn parse_hits_suffix_en_x_plus() {
        assert_eq!(parse_hits_suffix("X+2"), Some(HitCount::XPlus(2)));
    }

    #[test]
    fn parse_hits_suffix_en_x_plus_space() {
        assert_eq!(parse_hits_suffix("X + 3"), Some(HitCount::XPlus(3)));
    }

    #[test]
    fn parse_hits_suffix_en_x_times() {
        assert_eq!(parse_hits_suffix("X times"), Some(HitCount::XTimes));
    }

    #[test]
    fn parse_hits_suffix_en_lowercase_x_times() {
        assert_eq!(parse_hits_suffix("x times"), Some(HitCount::XTimes));
    }

    #[test]
    fn parse_hits_suffix_en_fixed_times() {
        assert_eq!(parse_hits_suffix("3 times"), Some(HitCount::Fixed(3)));
    }

    #[test]
    fn parse_hits_suffix_x_plus_no_digits() {
        assert_eq!(parse_hits_suffix("X+abc"), None);
    }

    #[test]
    fn parse_hits_suffix_times_no_digit_before() {
        assert_eq!(parse_hits_suffix(" times "), None);
    }

    #[test]
    fn parse_hits_suffix_no_pattern() {
        assert_eq!(parse_hits_suffix("just some text"), None);
    }

    #[test]
    fn parse_damage_multiplier_hits_invalid_desc() {
        assert_eq!(parse_damage_multiplier_hits(""), None);
        assert_eq!(parse_damage_multiplier_hits("  "), None);
    }

    #[test]
    fn parse_damage_multiplier_hits_zero_damage() {
        assert_eq!(parse_damage_multiplier_hits("Deal 0 damage."), None);
    }

    #[test]
    fn parse_damage_multiplier_hits_overflow_damage() {
        assert_eq!(parse_damage_multiplier_hits("Deal 1001 damage."), None);
    }

    #[test]
    fn parse_damage_multiplier_hits_en_strike() {
        assert_eq!(
            parse_damage_multiplier_hits("Deal 6 damage."),
            Some((6, HitCount::Fixed(1)))
        );
    }

    #[test]
    fn parse_damage_multiplier_hits_en_multi_hit() {
        assert_eq!(
            parse_damage_multiplier_hits("Deal 3 damage 4 times. Exhaust."),
            Some((3, HitCount::Fixed(4)))
        );
    }

    #[test]
    fn parse_target_type_en_aoe() {
        assert_eq!(
            parse_target_type("deal damage to all enemies", &en_locale()),
            TargetType::AoE
        );
    }

    #[test]
    fn parse_target_type_en_aoe_caps() {
        assert_eq!(
            parse_target_type("deal damage to ALL enemies", &en_locale()),
            TargetType::AoE
        );
    }

    #[test]
    fn parse_target_type_en_random() {
        assert_eq!(
            parse_target_type("deal damage to a random enemy", &en_locale()),
            TargetType::RandomTarget
        );
    }

    #[test]
    fn parse_target_type_en_targeted() {
        assert_eq!(
            parse_target_type("deal damage", &en_locale()),
            TargetType::Targeted
        );
    }

    #[test]
    fn parse_target_type_ja_aoe() {
        assert_eq!(
            parse_target_type("すべての敵にダメージを与える", &ja_locale()),
            TargetType::AoE
        );
    }

    #[test]
    fn parse_target_type_ko_random() {
        assert_eq!(
            parse_target_type("무작위 적에게 피해를 줍니다", &ko_locale()),
            TargetType::RandomTarget
        );
    }

    #[test]
    fn parse_damage_non_attack() {
        assert_eq!(parse_damage("Deal 6 damage", "SKILL", &en_locale()), None);
        assert_eq!(parse_damage("Deal 6 damage", "POWER", &en_locale()), None);
        assert_eq!(parse_damage("Deal 6 damage", "CURSE", &en_locale()), None);
    }

    #[test]
    fn parse_damage_non_attack_zh() {
        assert_eq!(parse_damage("造成 6 点伤害。", "SKILL", &zh_locale()), None);
    }

    #[test]
    fn parse_energy_gain_chinese_char() {
        assert_eq!(super::parse_energy_gain("获得 能量 能量 。"), 2);
    }

    #[test]
    fn parse_energy_gain_zero() {
        assert_eq!(super::parse_energy_gain("no energy here"), 0);
    }

    #[test]
    fn parse_strength_gain_en() {
        let c = skill("Gain 3 Strength.");
        let e = parse_card_effect(&c, &en_locale()).unwrap();
        assert_eq!(e.strength_gain, 3);
    }

    #[test]
    fn parse_strength_gain_no_match() {
        assert_eq!(
            super::parse_strength_gain("no strength here.", &en_locale()),
            0
        );
    }

    #[test]
    fn parse_strength_gain_zh_loss_skipped() {
        let c = skill("失去 3 点 力量 。");
        let e = parse_card_effect(&c, &zh_locale());
        assert!(e.is_none() || e.unwrap().strength_gain == 0);
    }

    #[test]
    fn parse_strength_gain_en_loss_skipped() {
        let c = skill("lose 3 Strength.");
        let e = parse_card_effect(&c, &en_locale());
        assert!(e.is_none() || e.unwrap().strength_gain == 0);
    }

    #[test]
    fn parse_vulnerable_en_with_amount() {
        let c = skill("Apply 2 Vulnerable.");
        let e = parse_card_effect(&c, &en_locale()).unwrap();
        assert_eq!(e.vulnerable, Some(2));
    }

    #[test]
    fn parse_vulnerable_en_default_one() {
        let c = skill("Apply Vulnerable.");
        let e = parse_card_effect(&c, &en_locale()).unwrap();
        assert_eq!(e.vulnerable, Some(1));
    }

    #[test]
    fn parse_vulnerable_no_match() {
        assert_eq!(super::parse_vulnerable("just a skill", &en_locale()), None);
    }

    #[test]
    fn parse_stance_en_wrath() {
        assert_eq!(
            parse_stance("Enter Wrath.", &en_locale()),
            StanceEffect::EnterWrath
        );
    }

    #[test]
    fn parse_stance_en_calm() {
        assert_eq!(
            parse_stance("Enter Calm.", &en_locale()),
            StanceEffect::EnterCalm
        );
    }

    #[test]
    fn parse_stance_en_exit() {
        assert_eq!(
            parse_stance("Exit your stance.", &en_locale()),
            StanceEffect::ExitStance
        );
    }

    #[test]
    fn parse_stance_en_divinity() {
        assert_eq!(
            parse_stance("Enter Divinity.", &en_locale()),
            StanceEffect::EnterDivinity
        );
    }

    #[test]
    fn parse_stance_ja_wrath() {
        assert_eq!(
            parse_stance("憤怒に 入る 。", &ja_locale()),
            StanceEffect::EnterWrath
        );
    }

    #[test]
    fn parse_stance_ja_calm() {
        assert_eq!(
            parse_stance("平静に 入る 。", &ja_locale()),
            StanceEffect::EnterCalm
        );
    }

    #[test]
    fn parse_stance_ja_exit() {
        assert_eq!(
            parse_stance("構え を 解除 する。", &ja_locale()),
            StanceEffect::ExitStance
        );
    }

    #[test]
    fn parse_stance_ja_divinity() {
        assert_eq!(
            parse_stance("神格に 入る 。", &ja_locale()),
            StanceEffect::EnterDivinity
        );
    }

    #[test]
    fn parse_stance_ko_wrath() {
        assert_eq!(
            parse_stance("분노에 들어갑니다 。", &ko_locale()),
            StanceEffect::EnterWrath
        );
    }

    #[test]
    fn parse_stance_ko_calm() {
        assert_eq!(
            parse_stance("평온에 들어갑니다 。", &ko_locale()),
            StanceEffect::EnterCalm
        );
    }

    #[test]
    fn parse_stance_ko_exit() {
        assert_eq!(
            parse_stance("자세를 해제합니다 。", &ko_locale()),
            StanceEffect::ExitStance
        );
    }

    #[test]
    fn parse_stance_ko_divinity() {
        assert_eq!(
            parse_stance("신격에 들어갑니다 。", &ko_locale()),
            StanceEffect::EnterDivinity
        );
    }

    #[test]
    fn parse_stance_none() {
        assert_eq!(
            parse_stance("just a normal card.", &en_locale()),
            StanceEffect::None
        );
    }

    #[test]
    fn parse_mantra_en_with_amount() {
        assert_eq!(super::parse_mantra("Gain 2 Mantra.", &en_locale()), 2);
    }

    #[test]
    fn parse_mantra_en_default_one() {
        assert_eq!(super::parse_mantra("Gain Mantra.", &en_locale()), 1);
    }

    #[test]
    fn parse_mantra_no_match() {
        assert_eq!(super::parse_mantra("just a skill", &en_locale()), 0);
    }

    #[test]
    fn parse_execute_en_fallback() {
        let custom_locale = EffectParserLocale {
            execute_keywords: vec![],
            ..en_locale()
        };
        assert_eq!(
            super::parse_execute("If HP is 15 or less, set its HP to 0.", &custom_locale),
            Some(15)
        );
    }

    #[test]
    fn parse_execute_en_hp_is() {
        let c = skill("If HP is 10, set its HP to 0.");
        let e = parse_card_effect(&c, &en_locale()).unwrap();
        assert_eq!(e.execute, Some(10));
    }

    #[test]
    fn parse_execute_no_match() {
        assert_eq!(super::parse_execute("no execute here.", &en_locale()), None);
    }

    #[test]
    fn parse_exhaust_zh_non_attacks() {
        assert_eq!(
            parse_exhaust("消耗 所有非攻击牌。", &zh_locale()),
            ExhaustKind::NonAttacks
        );
    }

    #[test]
    fn parse_exhaust_zh_attacks() {
        assert_eq!(
            parse_exhaust("消耗 所有攻击牌。", &zh_locale()),
            ExhaustKind::Attacks
        );
    }

    #[test]
    fn parse_exhaust_zh_random() {
        assert_eq!(
            parse_exhaust("消耗 随机 一张牌。", &zh_locale()),
            ExhaustKind::Random
        );
    }

    #[test]
    fn parse_exhaust_zh_chosen_select() {
        assert_eq!(
            parse_exhaust("消耗 选择 的一张牌。", &zh_locale()),
            ExhaustKind::Chosen
        );
    }

    #[test]
    fn parse_exhaust_zh_chosen_one_card() {
        assert_eq!(
            parse_exhaust("消耗 一张牌。", &zh_locale()),
            ExhaustKind::Chosen
        );
    }

    #[test]
    fn parse_exhaust_en_self() {
        assert_eq!(parse_exhaust("Exhaust.", &en_locale()), ExhaustKind::Self_);
    }

    #[test]
    fn parse_exhaust_en_all() {
        assert_eq!(
            parse_exhaust("Exhaust all cards.", &en_locale()),
            ExhaustKind::All
        );
    }

    #[test]
    fn parse_exhaust_en_non_attacks() {
        assert_eq!(
            parse_exhaust("Exhaust all non-attack cards.", &en_locale()),
            ExhaustKind::NonAttacks
        );
    }

    #[test]
    fn parse_exhaust_en_attacks() {
        assert_eq!(
            parse_exhaust("Exhaust all attack cards.", &en_locale()),
            ExhaustKind::Attacks
        );
    }

    #[test]
    fn parse_exhaust_en_random() {
        assert_eq!(
            parse_exhaust("Exhaust a random card.", &en_locale()),
            ExhaustKind::Random
        );
    }

    #[test]
    fn parse_exhaust_en_chosen() {
        assert_eq!(
            parse_exhaust("Choose a card to Exhaust.", &en_locale()),
            ExhaustKind::Chosen
        );
    }

    #[test]
    fn parse_exhaust_no_match() {
        assert_eq!(
            parse_exhaust("just a skill", &en_locale()),
            ExhaustKind::None
        );
    }

    #[test]
    fn parse_x_cost_zh_spend_all() {
        let c = skill("花费所有 [E] 。获得 X 点 力量 。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert!(e.x_cost);
    }

    #[test]
    fn parse_x_cost_en_spend_all() {
        let c = skill("Spend all [E]. Gain X Strength.");
        let e = parse_card_effect(&c, &en_locale()).unwrap();
        assert!(e.x_cost);
    }

    #[test]
    fn is_scan_relevant_all_false() {
        let e = CardEffect {
            damage: None,
            energy_gain: 0,
            strength_gain: 0,
            vulnerable: None,
            stance: StanceEffect::None,
            mantra_gain: 0,
            execute: None,
            exhaust: ExhaustKind::None,
            x_cost: false,
        };
        assert!(!is_scan_relevant(&e));
    }

    #[test]
    fn is_scan_relevant_damage() {
        let e = CardEffect {
            damage: Some(DamageEffect {
                amount: 6,
                hits: HitCount::Fixed(1),
                target_type: TargetType::Targeted,
            }),
            energy_gain: 0,
            strength_gain: 0,
            vulnerable: None,
            stance: StanceEffect::None,
            mantra_gain: 0,
            execute: None,
            exhaust: ExhaustKind::None,
            x_cost: false,
        };
        assert!(is_scan_relevant(&e));
    }

    #[test]
    fn is_scan_relevant_energy_gain() {
        let e = CardEffect {
            damage: None,
            energy_gain: 2,
            strength_gain: 0,
            vulnerable: None,
            stance: StanceEffect::None,
            mantra_gain: 0,
            execute: None,
            exhaust: ExhaustKind::None,
            x_cost: false,
        };
        assert!(is_scan_relevant(&e));
    }

    #[test]
    fn is_scan_relevant_strength_gain() {
        let e = CardEffect {
            damage: None,
            energy_gain: 0,
            strength_gain: 3,
            vulnerable: None,
            stance: StanceEffect::None,
            mantra_gain: 0,
            execute: None,
            exhaust: ExhaustKind::None,
            x_cost: false,
        };
        assert!(is_scan_relevant(&e));
    }

    #[test]
    fn is_scan_relevant_vulnerable() {
        let e = CardEffect {
            damage: None,
            energy_gain: 0,
            strength_gain: 0,
            vulnerable: Some(2),
            stance: StanceEffect::None,
            mantra_gain: 0,
            execute: None,
            exhaust: ExhaustKind::None,
            x_cost: false,
        };
        assert!(is_scan_relevant(&e));
    }

    #[test]
    fn is_scan_relevant_stance() {
        let e = CardEffect {
            damage: None,
            energy_gain: 0,
            strength_gain: 0,
            vulnerable: None,
            stance: StanceEffect::EnterWrath,
            mantra_gain: 0,
            execute: None,
            exhaust: ExhaustKind::None,
            x_cost: false,
        };
        assert!(is_scan_relevant(&e));
    }

    #[test]
    fn is_scan_relevant_mantra() {
        let e = CardEffect {
            damage: None,
            energy_gain: 0,
            strength_gain: 0,
            vulnerable: None,
            stance: StanceEffect::None,
            mantra_gain: 2,
            execute: None,
            exhaust: ExhaustKind::None,
            x_cost: false,
        };
        assert!(is_scan_relevant(&e));
    }

    #[test]
    fn is_scan_relevant_execute() {
        let e = CardEffect {
            damage: None,
            energy_gain: 0,
            strength_gain: 0,
            vulnerable: None,
            stance: StanceEffect::None,
            mantra_gain: 0,
            execute: Some(30),
            exhaust: ExhaustKind::None,
            x_cost: false,
        };
        assert!(is_scan_relevant(&e));
    }

    #[test]
    fn is_scan_relevant_exhaust() {
        let e = CardEffect {
            damage: None,
            energy_gain: 0,
            strength_gain: 0,
            vulnerable: None,
            stance: StanceEffect::None,
            mantra_gain: 0,
            execute: None,
            exhaust: ExhaustKind::Self_,
            x_cost: false,
        };
        assert!(is_scan_relevant(&e));
    }

    #[test]
    fn is_scan_relevant_x_cost() {
        let e = CardEffect {
            damage: None,
            energy_gain: 0,
            strength_gain: 0,
            vulnerable: None,
            stance: StanceEffect::None,
            mantra_gain: 0,
            execute: None,
            exhaust: ExhaustKind::None,
            x_cost: true,
        };
        assert!(is_scan_relevant(&e));
    }

    #[test]
    fn parse_card_effect_en_strike() {
        let c = card("Deal 6 damage.");
        let e = parse_card_effect(&c, &en_locale()).unwrap();
        let dmg = e.damage.as_ref().unwrap();
        assert_eq!(dmg.amount, 6);
        assert_eq!(dmg.target_type, TargetType::Targeted);
    }

    #[test]
    fn parse_card_effect_en_aoe() {
        let c = card("Deal 8 damage to all enemies.");
        let e = parse_card_effect(&c, &en_locale()).unwrap();
        let dmg = e.damage.as_ref().unwrap();
        assert_eq!(dmg.amount, 8);
        assert_eq!(dmg.target_type, TargetType::AoE);
    }

    #[test]
    fn parse_mantra_ja() {
        let c = skill("2 マントラ を得る。");
        let e = parse_card_effect(&c, &ja_locale()).unwrap();
        assert_eq!(e.mantra_gain, 2);
    }

    #[test]
    fn parse_vulnerable_ja() {
        let c = skill("敵に 2 脆弱 を与える。");
        let e = parse_card_effect(&c, &ja_locale()).unwrap();
        assert_eq!(e.vulnerable, Some(2));
    }

    #[test]
    fn parse_strength_gain_ja() {
        let c = skill("3 筋力 を得る。");
        let e = parse_card_effect(&c, &ja_locale()).unwrap();
        assert_eq!(e.strength_gain, 3);
    }

    #[test]
    fn parse_execute_ja() {
        let c = skill("HPが15なら0にする。");
        let e = parse_card_effect(&c, &ja_locale()).unwrap();
        assert_eq!(e.execute, Some(15));
    }

    #[test]
    fn parse_exhaust_ja_self() {
        let c = skill("廃棄 する。");
        let e = parse_card_effect(&c, &ja_locale()).unwrap();
        assert_eq!(e.exhaust, ExhaustKind::Self_);
    }

    #[test]
    fn parse_mantra_ko() {
        let c = skill("만트라 2 를 얻습니다。");
        let e = parse_card_effect(&c, &ko_locale()).unwrap();
        assert_eq!(e.mantra_gain, 2);
    }

    #[test]
    fn parse_vulnerable_ko() {
        let c = skill("적에게 취약 2 를 줍니다。");
        let e = parse_card_effect(&c, &ko_locale()).unwrap();
        assert_eq!(e.vulnerable, Some(2));
    }

    #[test]
    fn parse_strength_gain_ko() {
        let c = skill("힘 3 을 얻습니다。");
        let e = parse_card_effect(&c, &ko_locale()).unwrap();
        assert_eq!(e.strength_gain, 3);
    }

    #[test]
    fn parse_execute_ko() {
        let c = skill("HP가 15 이하라면 체력을 0으로 만듭니다。");
        let e = parse_card_effect(&c, &ko_locale()).unwrap();
        assert_eq!(e.execute, Some(15));
    }

    #[test]
    fn parse_exhaust_ko_self() {
        let c = skill("소멸 됩니다。");
        let e = parse_card_effect(&c, &ko_locale()).unwrap();
        assert_eq!(e.exhaust, ExhaustKind::Self_);
    }

    #[test]
    fn parse_x_cost_zh_x_in_desc() {
        let c = skill("造成 X 点伤害。");
        let e = parse_card_effect(&c, &zh_locale()).unwrap();
        assert!(e.x_cost);
    }

    #[test]
    fn parse_card_effect_power_rejected() {
        let c = power("At the start of your turn, gain 2 Strength.");
        let e = parse_card_effect(&c, &en_locale()).unwrap();
        assert_eq!(e.strength_gain, 2);
    }
}
