use super::helpers::{contains_all, extract_first_integer};

pub(super) fn parse_str_gain(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if !segment.contains("敌人") && contains_all(segment, &["获得", "力量"]) {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        if segment.contains("Gain") && segment.contains("Strength") {
            return extract_first_integer(segment);
        }
    }
    None
}

pub(super) fn parse_dex_gain(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if contains_all(segment, &["获得", "敏捷"]) {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        if segment.contains("Gain") && segment.contains("Dexterity") {
            return extract_first_integer(segment);
        }
    }
    None
}

pub(super) fn parse_poison(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if segment.contains("中毒") {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("poison") && !lower.contains("no.") {
            return extract_first_integer(segment);
        }
    }
    None
}

pub(super) fn parse_vulnerable(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if segment.contains("易伤") {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("vulnerable") {
            return extract_first_integer(segment);
        }
    }
    None
}

pub(super) fn parse_weak(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if segment.contains("虚弱") {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("weak") {
            return extract_first_integer(segment);
        }
    }
    None
}

pub(super) fn parse_energy_gain(desc: &str) -> Option<i64> {
    let count = desc.matches("[E]").count() as i64;
    if count > 0 { Some(count) } else { None }
}

pub(super) fn parse_str_loss(desc: &str) -> (Option<i64>, bool) {
    for segment in desc.split('。') {
        if segment.contains("敌人") && segment.contains("失去") && segment.contains("力量") {
            let amount = extract_first_integer(segment);
            let temporary = segment.contains("一回合") || desc.contains("一回合");
            return (amount, temporary);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("enemy") && lower.contains("lose") && lower.contains("strength") {
            let amount = extract_first_integer(segment);
            let temporary = lower.contains("1 turn") || desc.to_lowercase().contains("1 turn");
            return (amount, temporary);
        }
    }
    (None, false)
}

pub(super) fn parse_mantra(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if segment.contains("真言") {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("mantra") {
            return extract_first_integer(segment);
        }
    }
    None
}

pub(super) fn parse_focus_gain(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if segment.contains("集中") {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("focus") {
            return extract_first_integer(segment);
        }
    }
    None
}
