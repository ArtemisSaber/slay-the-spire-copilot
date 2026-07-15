use serde_json::Value;

use super::{CardInfo, PotionInfo, PowerInfo, RelicInfo};

pub(super) fn extract_cards(items: &[Value]) -> Vec<CardInfo> {
    items.iter().map(CardInfo::from_json).collect()
}

pub(super) fn extract_powers(items: &[Value]) -> Vec<PowerInfo> {
    items
        .iter()
        .map(|power| PowerInfo {
            id: power
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string(),
            name: power
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("?")
                .to_string(),
            amount: power
                .get("amount")
                .and_then(|number| number.as_i64())
                .unwrap_or(0),
        })
        .collect()
}

pub(super) fn extract_card_names(items: &[Value]) -> Vec<String> {
    items
        .iter()
        .map(|card| {
            card.get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("?")
                .to_string()
        })
        .collect()
}

pub(super) fn first_array<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Vec<Value>> {
    keys.iter()
        .find_map(|key| value.get(key).and_then(|candidate| candidate.as_array()))
}

pub(super) fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(key)
            .and_then(|candidate| candidate.as_str())
            .map(|text| text.trim().to_string())
            .filter(|text| is_readable_text(text))
    })
}

pub(super) fn first_raw_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(key)
            .and_then(|candidate| candidate.as_str())
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
    })
}

pub(super) fn is_readable_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    let total = trimmed
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    if total == 0 {
        return false;
    }

    let question_marks = trimmed
        .chars()
        .filter(|character| *character == '?')
        .count();
    if question_marks >= 2 && question_marks * 3 >= total {
        return false;
    }

    trimmed.chars().any(|character| {
        character.is_alphabetic() || ('\u{4e00}'..='\u{9fff}').contains(&character)
    })
}

pub(super) fn extract_relic_infos(items: &[Value]) -> Vec<RelicInfo> {
    items
        .iter()
        .map(|relic| match relic {
            Value::String(name) => RelicInfo {
                id: String::new(),
                name: name.clone(),
                description: String::new(),
                counter: None,
                price: None,
            },
            Value::Object(_) => RelicInfo {
                id: relic
                    .get("id")
                    .and_then(|value| value.as_str())
                    .unwrap_or("?")
                    .to_string(),
                name: relic
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("?")
                    .to_string(),
                description: relic
                    .get("description")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_string(),
                counter: relic
                    .get("counter")
                    .and_then(|value| value.as_i64())
                    .filter(|counter| *counter >= 0),
                price: relic.get("price").and_then(|value| value.as_i64()),
            },
            _ => RelicInfo {
                id: String::new(),
                name: "?".to_string(),
                description: String::new(),
                counter: None,
                price: None,
            },
        })
        .collect()
}

pub(super) fn extract_potion_infos(items: &[Value]) -> Vec<PotionInfo> {
    items
        .iter()
        .enumerate()
        .filter(|(_, potion)| {
            potion
                .get("id")
                .and_then(|id| id.as_str())
                .map(|id| id != "Potion Slot")
                .unwrap_or(true)
        })
        .map(|(slot, potion)| PotionInfo {
            id: potion
                .get("id")
                .and_then(|value| value.as_str())
                .map(str::to_string),
            slot,
            name: potion
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("?")
                .to_string(),
            description: potion
                .get("description")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string(),
            price: potion.get("price").and_then(|value| value.as_i64()),
            can_use: potion
                .get("can_use")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            can_discard: potion
                .get("can_discard")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            requires_target: potion
                .get("requires_target")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
        })
        .collect()
}

pub(super) fn extract_event_choice(option: &Value) -> Option<String> {
    match option {
        Value::String(text) => {
            let text = text.trim();
            is_readable_text(text).then(|| text.to_string())
        }
        Value::Object(_) => first_string(
            option,
            &[
                "text",
                "description",
                "choice_text",
                "button_text",
                "label",
                "name",
            ],
        ),
        _ => None,
    }
}
