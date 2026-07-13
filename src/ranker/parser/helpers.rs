pub(super) use crate::parsing::extract_first_integer;

pub(super) static ORB_TYPES: &[(&str, &str)] = &[
    ("闪电", "Lightning"),
    ("Lightning", "Lightning"),
    ("冰霜", "Frost"),
    ("Frost", "Frost"),
    ("黑暗", "Dark"),
    ("Dark", "Dark"),
    ("等离子", "Plasma"),
    ("Plasma", "Plasma"),
];

pub(super) fn extract_last_integer(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    let mut last = None;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            let start = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            if let Some(parsed) = std::str::from_utf8(&bytes[start..index])
                .ok()
                .and_then(|value| {
                    value
                        .parse::<i64>()
                        .map_err(|error| tracing::warn!("integer parse failed: {error}"))
                        .ok()
                })
            {
                last = Some(parsed);
            }
        } else {
            index += 1;
        }
    }
    last
}

pub(super) fn extract_integer_before_keyword(text: &str, keyword: &str) -> Option<i64> {
    text.find(keyword)
        .and_then(|position| extract_last_integer(&text[..position]))
}

pub(super) fn contains_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| text.contains(pattern))
}

pub(super) fn contains_all(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().all(|pattern| text.contains(pattern))
}
