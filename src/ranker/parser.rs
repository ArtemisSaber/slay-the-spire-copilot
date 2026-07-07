#[derive(Debug, Clone, Default)]
pub struct ParsedEffects {
    pub damage: Option<i64>,
    pub hits: i64,
    pub block: Option<i64>,
    pub heal: Option<i64>,
    pub draw: Option<i64>,
    pub self_damage: Option<i64>,
    pub str_gain: Option<i64>,
    pub dex_gain: Option<i64>,
    pub poison: Option<i64>,
    pub vulnerable: Option<i64>,
    pub weak: Option<i64>,
    pub energy_gain: Option<i64>,
    pub str_loss: Option<i64>,
    pub str_loss_temp: bool,
    pub mantra: Option<i64>,
    pub focus_gain: Option<i64>,
    pub exhaust_count: i64,
    pub ethereal: bool,
    pub channel_orb: Option<String>,
    pub evoke_orb: Option<String>,
    pub orb_slot_expand: i64,
    pub exits_stance: bool,
    pub enters_wrath: bool,
    pub enters_calm: bool,
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
            return std::str::from_utf8(&bytes[start..i]).ok().and_then(|n| {
                n.parse::<i64>()
                    .map_err(|e| tracing::warn!("integer parse failed: {e}"))
                    .ok()
            });
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
            if let Some(parsed) = std::str::from_utf8(&bytes[start..i]).ok().and_then(|s| {
                s.parse::<i64>()
                    .map_err(|e| tracing::warn!("integer parse failed: {e}"))
                    .ok()
            }) {
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

fn contains_all(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().all(|p| text.contains(p))
}

static ORB_TYPES: &[(&str, &str)] = &[
    ("闪电", "Lightning"),
    ("Lightning", "Lightning"),
    ("冰霜", "Frost"),
    ("Frost", "Frost"),
    ("黑暗", "Dark"),
    ("Dark", "Dark"),
    ("等离子", "Plasma"),
    ("Plasma", "Plasma"),
];

pub fn parse_description(desc: &str) -> ParsedEffects {
    let damage = parse_damage_amount(desc);
    let hits = parse_hits(desc);
    let block = parse_block(desc);
    let heal = parse_heal(desc);
    let draw = parse_draw(desc);
    let self_damage = parse_self_damage(desc);
    let str_gain = parse_str_gain(desc);
    let dex_gain = parse_dex_gain(desc);
    let poison = parse_poison(desc);
    let vulnerable = parse_vulnerable(desc);
    let weak = parse_weak(desc);
    let energy_gain = parse_energy_gain(desc);
    let (str_loss, str_loss_temp) = parse_str_loss(desc);
    let mantra = parse_mantra(desc);
    let focus_gain = parse_focus_gain(desc);
    let exhaust_count = parse_exhaust(desc);
    let ethereal = parse_ethereal(desc);
    let channel_orb = parse_channel_orb(desc);
    let evoke_orb = parse_evoke_orb(desc);
    let orb_slot_expand = parse_orb_slot_expand(desc);
    let exits_stance = parse_exits_stance(desc);
    let enters_wrath = parse_enters_wrath(desc);
    let enters_calm = parse_enters_calm(desc);

    ParsedEffects {
        damage,
        hits,
        block,
        heal,
        draw,
        self_damage,
        str_gain,
        dex_gain,
        poison,
        vulnerable,
        weak,
        energy_gain,
        str_loss,
        str_loss_temp,
        mantra,
        focus_gain,
        exhaust_count,
        ethereal,
        channel_orb,
        evoke_orb,
        orb_slot_expand,
        exits_stance,
        enters_wrath,
        enters_calm,
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
    for segment in desc.split('。') {
        if segment.contains("两次") {
            return 2;
        }
        if segment.contains('次')
            && let Some(n) = extract_integer_before_keyword(segment, "次")
        {
            return n;
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("twice") {
            return 2;
        }
        if (lower.contains("times") || lower.contains("time"))
            && let Some(n) = extract_integer_before_keyword(&lower, "time")
        {
            return n;
        }
    }
    1
}

fn parse_block(desc: &str) -> Option<i64> {
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

fn parse_heal(desc: &str) -> Option<i64> {
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

fn parse_draw(desc: &str) -> Option<i64> {
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

fn parse_self_damage(desc: &str) -> Option<i64> {
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

fn parse_str_gain(desc: &str) -> Option<i64> {
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

fn parse_dex_gain(desc: &str) -> Option<i64> {
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

fn parse_poison(desc: &str) -> Option<i64> {
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

fn parse_vulnerable(desc: &str) -> Option<i64> {
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

fn parse_weak(desc: &str) -> Option<i64> {
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

fn parse_energy_gain(desc: &str) -> Option<i64> {
    let count = desc.matches("[E]").count() as i64;
    if count > 0 { Some(count) } else { None }
}

fn parse_str_loss(desc: &str) -> (Option<i64>, bool) {
    for segment in desc.split('。') {
        if segment.contains("敌人") && segment.contains("失去") && segment.contains("力量") {
            let amount = extract_first_integer(segment);
            let temp = segment.contains("一回合") || desc.contains("一回合");
            return (amount, temp);
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("enemy") && lower.contains("lose") && lower.contains("strength") {
            let amount = extract_first_integer(segment);
            let temp = lower.contains("1 turn") || desc.to_lowercase().contains("1 turn");
            return (amount, temp);
        }
    }
    (None, false)
}

fn parse_mantra(desc: &str) -> Option<i64> {
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

fn parse_focus_gain(desc: &str) -> Option<i64> {
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

fn parse_exhaust(desc: &str) -> i64 {
    let zh_marker = "消耗";

    if let Some(n) = extract_integer_before_keyword(desc, zh_marker) {
        return n;
    }
    if desc.contains(zh_marker) {
        return 1;
    }

    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("exhaust") {
            if let Some(n) = extract_integer_before_keyword(&lower, "card")
                && n > 0
            {
                return n;
            }
            return 1;
        }
    }
    0
}

fn parse_ethereal(desc: &str) -> bool {
    desc.contains("虚无") || desc.contains("Ethereal")
}

fn parse_channel_orb(desc: &str) -> Option<String> {
    for segment in desc.split('。') {
        if segment.contains("充能") {
            for (keyword, orb_id) in ORB_TYPES {
                if segment.contains(keyword) {
                    return Some(orb_id.to_string());
                }
            }
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("channel") {
            for (keyword, orb_id) in ORB_TYPES {
                if lower.contains(&keyword.to_lowercase()) {
                    return Some(orb_id.to_string());
                }
            }
        }
    }
    None
}

fn parse_evoke_orb(desc: &str) -> Option<String> {
    for segment in desc.split('。') {
        if segment.contains("激发") {
            for (keyword, orb_id) in ORB_TYPES {
                if segment.contains(keyword) {
                    return Some(orb_id.to_string());
                }
            }
        }
    }
    for segment in desc.split('.') {
        let lower = segment.to_lowercase();
        if lower.contains("evoke") {
            for (keyword, orb_id) in ORB_TYPES {
                if lower.contains(&keyword.to_lowercase()) {
                    return Some(orb_id.to_string());
                }
            }
        }
    }
    None
}

fn parse_orb_slot_expand(desc: &str) -> i64 {
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

fn parse_exits_stance(desc: &str) -> bool {
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

fn parse_enters_wrath(desc: &str) -> bool {
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

fn parse_enters_calm(desc: &str) -> bool {
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

#[cfg(test)]
#[path = "tests/parser_tests.rs"]
mod tests;
