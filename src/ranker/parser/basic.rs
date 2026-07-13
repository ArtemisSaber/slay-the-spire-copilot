use super::helpers::{
    contains_all, contains_any, extract_first_integer, extract_integer_before_keyword,
};

pub(super) fn parse_damage_amount(desc: &str) -> Option<i64> {
    let zh_patterns = ["造成", "点伤害"];
    for segment in desc.split('。') {
        if contains_any(segment, &zh_patterns) {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("deal") && lower.contains("damage") {
            return extract_first_integer(segment);
        }
    }
    None
}

pub(super) fn parse_hits(desc: &str) -> i64 {
    for segment in desc.split('。') {
        if segment.contains("两次") {
            return 2;
        }
        if segment.contains('次')
            && let Some(number) = extract_integer_before_keyword(segment, "次")
        {
            return number;
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("twice") {
            return 2;
        }
        if (lower.contains("times") || lower.contains("time"))
            && let Some(number) = extract_integer_before_keyword(&lower, "time")
        {
            return number;
        }
    }
    1
}

pub(super) fn parse_block(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if contains_all(segment, &["获得", "格挡"]) {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        if segment.contains("Gain") && segment.contains("Block") {
            return extract_first_integer(segment);
        }
    }
    None
}

pub(super) fn parse_heal(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if segment.contains("回复") && segment.contains("生命") {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("heal") && lower.contains("hp") {
            return extract_first_integer(segment);
        }
    }
    None
}

pub(super) fn parse_draw(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if segment.contains('抽') && segment.contains('张') {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("draw") && lower.contains("card") {
            return extract_first_integer(segment);
        }
    }
    None
}

pub(super) fn parse_self_damage(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if contains_all(segment, &["失去", "生命"]) {
            return extract_first_integer(segment);
        }
    }
    for segment in desc.split('.') {
        if segment.contains("Lose") && segment.contains("HP") {
            return extract_first_integer(segment);
        }
    }
    None
}
