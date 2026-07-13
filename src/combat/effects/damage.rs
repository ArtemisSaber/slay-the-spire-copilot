use crate::locales::EffectParserLocale;
use crate::parsing::extract_first_integer;

use super::{DamageEffect, HitCount, TargetType};

/// Sanity cap for parsed damage values — valid STS card damage never exceeds this.
const MAX_PARSED_DAMAGE: i16 = 1000;

pub(super) fn parse_hits_suffix(segment: &str) -> Option<HitCount> {
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

pub(super) fn parse_damage_multiplier_hits(desc: &str) -> Option<(i16, HitCount)> {
    for segment in desc.split('。') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let Some(dmg) = extract_first_integer(segment) else {
            continue;
        };
        if dmg <= 0 || dmg > MAX_PARSED_DAMAGE {
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
        if dmg <= 0 || dmg > MAX_PARSED_DAMAGE {
            continue;
        }
        if let Some(hits) = parse_hits_suffix(segment) {
            return Some((dmg, hits));
        }
        return Some((dmg, HitCount::Fixed(1)));
    }
    None
}

pub(super) fn parse_target_type(desc: &str, locale: &EffectParserLocale) -> TargetType {
    let lower = desc.to_lowercase();
    if locale
        .aoe_keywords
        .iter()
        .any(|keyword| lower.contains(&keyword.to_lowercase()))
    {
        TargetType::AoE
    } else if locale
        .random_keywords
        .iter()
        .any(|keyword| lower.contains(&keyword.to_lowercase()))
    {
        TargetType::RandomTarget
    } else {
        TargetType::Targeted
    }
}

pub(super) fn parse_damage(
    desc: &str,
    card_type: &str,
    locale: &EffectParserLocale,
) -> Option<DamageEffect> {
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
