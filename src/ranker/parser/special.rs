use super::helpers::{ORB_TYPES, extract_first_integer, extract_integer_before_keyword};

pub(super) fn parse_exhaust(desc: &str) -> i64 {
    let zh_marker = "消耗";
    if let Some(number) = extract_integer_before_keyword(desc, zh_marker) {
        return number;
    }
    if desc.contains(zh_marker) {
        return 1;
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("exhaust") {
            if let Some(number) = extract_integer_before_keyword(&lower, "card")
                && number > 0
            {
                return number;
            }
            return 1;
        }
    }
    0
}

pub(super) fn parse_ethereal(desc: &str) -> bool {
    desc.contains("虚无") || desc.contains("Ethereal")
}

pub(super) fn parse_channel_orb(desc: &str) -> Option<String> {
    parse_orb(desc, "充能", "channel")
}

pub(super) fn parse_evoke_orb(desc: &str) -> Option<String> {
    parse_orb(desc, "激发", "evoke")
}

fn parse_orb(desc: &str, zh_verb: &str, english_verb: &str) -> Option<String> {
    for segment in desc.split('。') {
        if segment.contains(zh_verb)
            && let Some(orb) = find_orb(segment)
        {
            return Some(orb.to_string());
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains(english_verb) {
            for (keyword, orb_id) in ORB_TYPES {
                if lower.contains(&keyword.to_lowercase()) {
                    return Some(orb_id.to_string());
                }
            }
        }
    }
    None
}

fn find_orb(segment: &str) -> Option<&'static str> {
    ORB_TYPES
        .iter()
        .find_map(|(keyword, orb_id)| segment.contains(keyword).then_some(*orb_id))
}

pub(super) fn parse_orb_slot_expand(desc: &str) -> i64 {
    for segment in desc.split('。') {
        if segment.contains("充能球栏位") {
            return extract_first_integer(segment).unwrap_or(0);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("orb slot") {
            return extract_first_integer(segment).unwrap_or(0);
        }
    }
    0
}

pub(super) fn parse_exits_stance(desc: &str) -> bool {
    for segment in desc.split('。') {
        if (segment.contains("退出") || segment.contains("结束")) && segment.contains("姿态")
        {
            return true;
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if (lower.contains("exit") || lower.contains("end")) && lower.contains("stance") {
            return true;
        }
    }
    false
}

pub(super) fn parse_enters_wrath(desc: &str) -> bool {
    for segment in desc.split('。') {
        if segment.contains("进入") && segment.contains("愤怒") {
            return true;
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("enter") && lower.contains("wrath") {
            return true;
        }
    }
    false
}

pub(super) fn parse_enters_calm(desc: &str) -> bool {
    for segment in desc.split('。') {
        if segment.contains("进入") && segment.contains("宁静") {
            return true;
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("enter") && lower.contains("calm") {
            return true;
        }
    }
    false
}
