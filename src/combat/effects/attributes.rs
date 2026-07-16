use crate::locales::EffectParserLocale;
use crate::parsing::extract_first_integer;

use super::{ExhaustKind, StanceEffect};

fn has_unsupported_condition_or_timing(segment: &str) -> bool {
    let lower = segment.to_lowercase();
    let english = [
        "if ",
        "when ",
        "whenever",
        "next turn",
        "at the start of",
        "at start of",
        "for each",
    ];
    let localized = [
        "如果",
        "若",
        "每当",
        "下一回合",
        "下回合",
        "回合开始",
        "每消耗",
        "場合",
        "たび",
        "次のターン",
        "開始時",
        "경우",
        "때",
        "마다",
        "다음 턴",
    ];

    english.iter().any(|marker| lower.contains(marker))
        || localized.iter().any(|marker| segment.contains(marker))
}

fn has_strength_gain_verb(segment: &str) -> bool {
    let lower = segment.to_lowercase();
    lower.contains("gain")
        || segment.contains("获得")
        || segment.contains("得到")
        || segment.contains("得る")
        || segment.contains("獲得")
        || segment.contains("얻")
        || segment.contains("획득")
}

pub(super) fn parse_energy_gain(desc: &str) -> i16 {
    desc.split(['。', '.'])
        .filter(|segment| !has_unsupported_condition_or_timing(segment))
        .map(|segment| {
            segment.matches("[E]").count() as i16 + segment.matches("能量").count() as i16
        })
        .sum()
}

pub(super) fn parse_strength_gain(desc: &str, locale: &EffectParserLocale) -> i16 {
    let loss = &locale.strength_lose_keyword;
    for segment in desc.split(['。', '.']) {
        let segment = segment.trim();
        let lower = segment.to_lowercase();
        let mentions_strength =
            lower.contains("strength") || segment.contains(&locale.strength_gain_keyword);
        if !mentions_strength
            || !has_strength_gain_verb(segment)
            || has_unsupported_condition_or_timing(segment)
            || lower.contains(&loss.to_lowercase())
        {
            continue;
        }
        if let Some(n) = extract_first_integer(segment) {
            return n;
        }
    }
    0
}

pub(super) fn parse_vulnerable(desc: &str, locale: &EffectParserLocale) -> Option<i16> {
    let keyword = &locale.vulnerable_keyword;
    for segment in desc.split(['。', '.']) {
        let segment = segment.trim();
        if has_unsupported_condition_or_timing(segment) {
            continue;
        }
        if segment.to_lowercase().contains("vulnerable") || segment.contains(keyword) {
            let n = extract_first_integer(segment).unwrap_or(1);
            return Some(n);
        }
    }
    None
}

pub(super) fn parse_stance(desc: &str, locale: &EffectParserLocale) -> StanceEffect {
    for segment in desc.split(['。', '.']) {
        if has_unsupported_condition_or_timing(segment) {
            continue;
        }
        let lower = segment.to_lowercase();
        let matches_all = |keywords: &[String]| {
            keywords
                .iter()
                .all(|keyword| lower.contains(&keyword.to_lowercase()))
        };

        if matches_all(&locale.enter_wrath_keywords) {
            return StanceEffect::EnterWrath;
        }
        if matches_all(&locale.enter_calm_keywords) {
            return StanceEffect::EnterCalm;
        }
        if matches_all(&locale.exit_stance_keywords) {
            return StanceEffect::ExitStance;
        }
        if matches_all(&locale.enter_divinity_keywords) {
            return StanceEffect::EnterDivinity;
        }
    }
    StanceEffect::None
}

pub(super) fn parse_mantra(desc: &str, locale: &EffectParserLocale) -> i16 {
    let keyword = &locale.mantra_keyword;
    for segment in desc.split(['。', '.']) {
        if has_unsupported_condition_or_timing(segment) {
            continue;
        }
        if segment.contains(keyword) || segment.to_lowercase().contains("mantra") {
            return extract_first_integer(segment).unwrap_or(1);
        }
    }
    0
}

pub(super) fn parse_execute(desc: &str, locale: &EffectParserLocale) -> Option<i16> {
    let lower = desc.to_lowercase();
    if locale
        .execute_keywords
        .iter()
        .any(|keyword| lower.contains(&keyword.to_lowercase()))
    {
        return extract_first_integer(desc);
    }
    if lower.contains("hp is") || lower.contains("hp or less") || lower.contains("set its hp to 0")
    {
        return extract_first_integer(desc);
    }
    None
}

pub(super) fn parse_exhaust(desc: &str, locale: &EffectParserLocale) -> ExhaustKind {
    let keyword = &locale.exhaust_keyword;
    let lower = desc.to_lowercase();
    let has_exhaust = lower.contains(&keyword.to_lowercase()) || lower.contains("exhaust");
    if !has_exhaust {
        return ExhaustKind::None;
    }
    if (lower.contains("每消耗一张") || lower.contains("for each exhausted card"))
        && (lower.contains("所有手牌") || lower.contains("all cards"))
    {
        ExhaustKind::ExhaustAllDamagePerCard
    } else if lower.contains("所有非攻击牌") || lower.contains("all non-attack") {
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
        || (lower.contains("一张牌") && lower.contains(&keyword.to_lowercase()))
    {
        ExhaustKind::Chosen
    } else {
        ExhaustKind::Self_
    }
}
