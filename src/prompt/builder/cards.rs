use std::collections::HashMap;

use crate::locales::Locale;
use crate::state::CardInfo;

pub(crate) fn clean_description(raw: &str, locale: &Locale) -> String {
    let mut result = raw.to_string();
    result = result.replace('*', "");
    let energy: &str = &locale.monster.energy_token;
    for token in &["[R]", "[G]", "[B]", "[W]", "[E]"] {
        result = result.replace(token, energy);
    }
    for count in (2..=10).rev() {
        let pattern: String = (0..count).map(|_| energy).collect::<Vec<_>>().join(" ");
        let replacement = format!("{count} {energy}");
        result = result.replace(&pattern, &replacement);
    }
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    for punctuation in [".", ",", ";", "!", "?", "。", "、"] {
        result = result.replace(&format!(" {punctuation}"), punctuation);
    }
    result
}

pub(crate) fn format_card(card: &CardInfo, locale: &Locale) -> String {
    let card_type = match card.card_type.as_str() {
        "ATTACK" => &locale.i18n.type_attack,
        "SKILL" => &locale.i18n.type_skill,
        "POWER" => &locale.i18n.type_power,
        "CURSE" => &locale.i18n.type_curse,
        "STATUS" => &locale.i18n.type_status,
        _ => card.card_type.as_str(),
    };
    let mut result = locale
        .card
        .format
        .replace("{up}", if card.upgraded { "+" } else { "" })
        .replace("{name}", &card.name)
        .replace("{cost}", &card.cost.to_string())
        .replace("{type}", card_type);
    if !card.description.is_empty() {
        result.push_str(
            &locale
                .card
                .with_desc
                .replace("{desc}", &clean_description(&card.description, locale)),
        );
    }
    result
}

pub(crate) fn compact_pile(label: &str, cards: &[CardInfo], locale: &Locale) -> String {
    if cards.is_empty() {
        return locale.card.pile_empty.replace("{label}", label);
    }

    let mut name_counts: HashMap<&str, usize> = HashMap::new();
    for card in cards {
        *name_counts.entry(card.name.as_str()).or_insert(0) += 1;
    }

    let mut entries: Vec<String> = name_counts
        .into_iter()
        .map(|(name, count)| {
            if count == 1 {
                name.to_string()
            } else {
                format!("{name}×{count}")
            }
        })
        .collect();
    entries.sort();

    locale
        .card
        .pile
        .replace("{label}", label)
        .replace("{count}", &cards.len().to_string())
        .replace("{cards}", &entries.join(" "))
}

pub(crate) fn format_deck_section(cards: &[CardInfo], locale: &Locale) -> String {
    if cards.is_empty() {
        return String::new();
    }

    struct DeckEntry<'a> {
        card_type: &'a str,
        count: usize,
        cost: i64,
        upgraded: bool,
        name: &'a str,
        description: &'a str,
    }

    let mut aggregated: Vec<DeckEntry> = Vec::new();
    let mut seen: HashMap<(&str, &str), usize> = HashMap::new();
    for card in cards {
        let key = (card.card_type.as_str(), card.id.as_str());
        if let Some(index) = seen.get(&key) {
            aggregated[*index].count += 1;
        } else {
            seen.insert(key, aggregated.len());
            aggregated.push(DeckEntry {
                card_type: card.card_type.as_str(),
                count: 1,
                cost: card.cost,
                upgraded: card.upgraded,
                name: card.name.as_str(),
                description: card.description.as_str(),
            });
        }
    }

    let mut by_type: HashMap<&str, Vec<&DeckEntry>> = HashMap::new();
    for entry in &aggregated {
        by_type.entry(entry.card_type).or_default().push(entry);
    }
    for entries in by_type.values_mut() {
        entries.sort_by_key(|entry| entry.name);
    }

    let type_order = ["ATTACK", "SKILL", "POWER", "CURSE", "STATUS"];
    let mut lines: Vec<String> = vec![locale.card.deck_header.clone()];

    for &card_type in &type_order {
        if let Some(entries) = by_type.get(card_type) {
            let type_name = match card_type {
                "ATTACK" => &locale.i18n.type_attack,
                "SKILL" => &locale.i18n.type_skill,
                "POWER" => &locale.i18n.type_power,
                "CURSE" => &locale.i18n.type_curse,
                "STATUS" => &locale.i18n.type_status,
                _ => card_type,
            };
            let total: usize = entries.iter().map(|entry| entry.count).sum();
            lines.push(
                locale
                    .card
                    .type_group
                    .replace("{type}", type_name)
                    .replace("{count}", &total.to_string()),
            );
            for entry in entries {
                let description = if entry.description.is_empty() {
                    String::new()
                } else {
                    locale
                        .card
                        .with_desc
                        .replace("{desc}", &clean_description(entry.description, locale))
                };
                let prefix = if entry.upgraded { "+" } else { "" };
                if entry.count == 1 {
                    lines.push(format!(
                        "  {prefix}{}({cost}{suffix}){description}",
                        entry.name,
                        cost = entry.cost,
                        suffix = locale.card.cost_suffix,
                    ));
                } else {
                    let count_multiple = locale
                        .card
                        .card_count_multi
                        .replace("{count}", &entry.count.to_string());
                    lines.push(format!(
                        "  {prefix}{}({cost}{suffix}){count_multiple}{description}",
                        entry.name,
                        cost = entry.cost,
                        suffix = locale.card.cost_suffix,
                    ));
                }
            }
        }
    }

    lines.join("\n")
}
