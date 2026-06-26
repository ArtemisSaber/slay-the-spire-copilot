#[derive(Debug, Clone, Default)]
pub struct ParsedEffects {
    pub damage: Option<i64>,
    pub hits: i64,
    pub block: Option<i64>,
    pub self_damage: Option<i64>,
    pub exhaust_count: i64,
}

fn extract_first_integer(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            return std::str::from_utf8(&bytes[start..i])
                .ok()
                .and_then(|n| n.parse().ok());
        }
        i += 1;
    }
    None
}

fn extract_last_integer(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    let mut last = None;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if let Some(parsed) = std::str::from_utf8(&bytes[start..i])
                .ok()
                .and_then(|s| s.parse::<i64>().ok())
            {
                last = Some(parsed);
            }
        } else {
            i += 1;
        }
    }
    last
}

fn extract_integer_before_keyword(s: &str, keyword: &str) -> Option<i64> {
    if let Some(pos) = s.find(keyword) {
        let prefix = &s[..pos];
        extract_last_integer(prefix)
    } else {
        None
    }
}

fn contains_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}

pub fn parse_description(desc: &str) -> ParsedEffects {
    let damage = parse_damage_amount(desc);
    let hits = parse_hits(desc);
    let block = parse_block(desc);
    let self_damage = parse_self_damage(desc);
    let exhaust_count = parse_exhaust(desc);

    ParsedEffects {
        damage,
        hits,
        block,
        self_damage,
        exhaust_count,
    }
}

fn parse_damage_amount(desc: &str) -> Option<i64> {
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

fn parse_hits(desc: &str) -> i64 {
    let zh_marker = "次";
    let en_markers = ["times", "time"];

    for segment in desc.split('。') {
        if segment.contains(zh_marker)
            && let Some(n) = extract_integer_before_keyword(segment, zh_marker)
        {
            return n;
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        for marker in &en_markers {
            if lower.contains(*marker)
                && let Some(n) = extract_integer_before_keyword(&lower, marker)
            {
                return n;
            }
        }
    }
    1
}

fn parse_block(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if contains_any(segment, &["获得", "格挡"]) {
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

fn parse_self_damage(desc: &str) -> Option<i64> {
    for segment in desc.split('。') {
        if contains_any(segment, &["失去", "生命"]) {
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

fn parse_exhaust(desc: &str) -> i64 {
    let zh = "消耗";
    let en = "Exhaust";

    if desc.contains(zh) || desc.contains(en) {
        1
    } else {
        0
    }
}

#[cfg(test)]
#[path = "tests/parser_tests.rs"]
mod tests;
