use crate::locales::EffectParserLocale;
use crate::parsing::extract_first_integer;

use super::{ExhaustKind, StanceEffect};

pub(super) fn parse_energy_gain(desc: &str) -> i16 {
    desc.matches("[E]").count() as i16 + desc.matches("能量").count() as i16
}

pub(super) fn parse_strength_gain(desc: &str, locale: &EffectParserLocale) -> i16 {
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

pub(super) fn parse_vulnerable(desc: &str, locale: &EffectParserLocale) -> Option<i16> {
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

pub(super) fn parse_stance(desc: &str, locale: &EffectParserLocale) -> StanceEffect {
    let lower = desc.to_lowercase();

    let wrath_match = locale
        .enter_wrath_keywords
        .iter()
        .all(|keyword| lower.contains(&keyword.to_lowercase()));
    let calm_match = locale
        .enter_calm_keywords
        .iter()
        .all(|keyword| lower.contains(&keyword.to_lowercase()));
    let exit_match = locale
        .exit_stance_keywords
        .iter()
        .all(|keyword| lower.contains(&keyword.to_lowercase()));
    let divinity_match = locale
        .enter_divinity_keywords
        .iter()
        .all(|keyword| lower.contains(&keyword.to_lowercase()));

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

pub(super) fn parse_mantra(desc: &str, locale: &EffectParserLocale) -> i16 {
    let keyword = &locale.mantra_keyword;
    if desc.contains(keyword) || desc.to_lowercase().contains("mantra") {
        extract_first_integer(desc).unwrap_or(1)
    } else {
        0
    }
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
        || (lower.contains("一张牌") && lower.contains(&keyword.to_lowercase()))
    {
        ExhaustKind::Chosen
    } else {
        ExhaustKind::Self_
    }
}
