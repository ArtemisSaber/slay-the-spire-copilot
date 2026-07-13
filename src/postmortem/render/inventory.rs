use crate::locales::PostmortemLocale;
use serde_json::Value;
use std::collections::HashMap;

pub(super) fn append_inventory(
    report: &mut Vec<String>,
    state: Option<&Value>,
    pm: &PostmortemLocale,
) {
    let Some(state) = state else {
        return;
    };

    append_named_items(report, state, pm.section_relics.clone(), "relics");
    append_named_items(report, state, pm.section_potions.clone(), "potions");
    append_deck(report, state, pm);
}

fn append_named_items(report: &mut Vec<String>, state: &Value, section: String, key: &str) {
    let Some(items) = state.get(key).and_then(|value| value.as_array()) else {
        return;
    };
    if items.is_empty() {
        return;
    }

    report.push(String::new());
    report.push(section);
    for item in items {
        if let Some(name) = item.get("name").and_then(|value| value.as_str()) {
            let description = item
                .get("description")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            report.push(format!("- {name}: {description}"));
        }
    }
}

fn append_deck(report: &mut Vec<String>, state: &Value, pm: &PostmortemLocale) {
    let Some(deck_names) = state.get("deck_names").and_then(|value| value.as_array()) else {
        return;
    };
    if deck_names.is_empty() {
        return;
    }

    let mut cards: Vec<String> = deck_names
        .iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect();
    cards.sort();
    cards.dedup();
    let mut counts = HashMap::new();
    for card in deck_names.iter().filter_map(|value| value.as_str()) {
        *counts.entry(card.to_string()).or_insert(0) += 1;
    }

    report.push(String::new());
    report.push(pm.section_deck.clone());
    for name in &cards {
        let count = counts.get(name).copied().unwrap_or(1);
        if count > 1 {
            report.push(format!("- {name} ×{count}"));
        } else {
            report.push(format!("- {name}"));
        }
    }
}
