use std::collections::HashMap;

use crate::locales::Locale;
use crate::state::NormalizedState;

use super::cards::{clean_description, format_card, format_deck_section};
use super::inventory::build_relics_potions_section;
use super::monsters::build_monsters_section;
use super::status::status_line;

pub(crate) fn build_shop(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: shop]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    if !state.master_cards.is_empty() {
        lines.push(format_deck_section(&state.master_cards, locale));
    }

    if !state.shop_cards.is_empty()
        || !state.shop_relics.is_empty()
        || !state.shop_potions.is_empty()
        || state.purge_available
    {
        lines.push(locale.sections.shop.clone());
    }

    if !state.shop_cards.is_empty() {
        lines.push(locale.sections.shop_cards.clone());
        for (index, card) in state.shop_cards.iter().enumerate() {
            let label = (b'A' + index as u8) as char;
            let price = card
                .price
                .map_or("?".to_string(), |value| value.to_string());
            lines.push(format!(
                "{label}. {}  ({} gold)",
                format_card(card, locale),
                price
            ));
        }
        lines.push(String::new());
    }

    if !state.shop_relics.is_empty() {
        lines.push(locale.sections.shop_relics.clone());
        for (index, relic) in state.shop_relics.iter().enumerate() {
            let label = (b'A' + index as u8) as char;
            let price = relic
                .price
                .map_or("?".to_string(), |value| value.to_string());
            lines.push(format!(
                "{label}. {} — {}  ({} gold)",
                relic.name,
                clean_description(&relic.description, locale),
                price
            ));
        }
        lines.push(String::new());
    }

    if !state.shop_potions.is_empty() {
        lines.push(locale.sections.shop_potions.clone());
        for (index, potion) in state.shop_potions.iter().enumerate() {
            let label = (b'A' + index as u8) as char;
            let price = potion
                .price
                .map_or("?".to_string(), |value| value.to_string());
            lines.push(format!(
                "{label}. {} — {}  ({} gold)",
                potion.name,
                clean_description(&potion.description, locale),
                price
            ));
        }
        lines.push(String::new());
    }

    if state.purge_available {
        lines.push(locale.sections.shop_purge.clone());
        let cost = state
            .purge_cost
            .map_or("unknown".to_string(), |value| value.to_string());
        lines.push(format!("Remove a card for {} gold", cost));
        lines.push(format!("Deck: {}", state.deck_names.join(", ")));
        lines.push(String::new());
    }

    lines.join("\n")
}

pub(crate) fn build_event_choice(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: event_choice]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    if state.event_name.is_some() || state.event_id.is_some() || state.room_type.is_some() {
        lines.push(locale.sections.event.clone());
        if let Some(name) = &state.event_name {
            lines.push(name.clone());
        }
        if let Some(id) = &state.event_id {
            lines.push(locale.warnings.event_id.replace("{id}", id));
        }
        if state.event_name.is_none()
            && state.event_id.is_none()
            && let Some(room_type) = &state.room_type
        {
            lines.push(
                locale
                    .warnings
                    .event_unreadable
                    .replace("{room_type}", room_type.as_str()),
            );
        }
    }
    if let Some(body) = &state.event_body {
        lines.push(body.clone());
        lines.push(String::new());
    }

    if !state.event_choices.is_empty() {
        lines.push(locale.sections.options.clone());
        for (index, choice) in state.event_choices.iter().enumerate() {
            let label = (b'A' + index as u8) as char;
            lines.push(format!("{label}. {choice}"));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

pub(crate) fn build_generic(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: generic]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    if !state.monsters.is_empty() {
        lines.push(build_monsters_section(state, locale));
    }

    if !state.hand_cards.is_empty() {
        let mut name_counts: HashMap<&str, usize> = HashMap::new();
        for card in &state.hand_cards {
            *name_counts.entry(card.name.as_str()).or_insert(0) += 1;
        }
        let cards: Vec<String> = name_counts
            .into_iter()
            .map(|(name, count)| {
                if count == 1 {
                    name.to_string()
                } else {
                    format!("{name}×{count}")
                }
            })
            .collect();
        lines.push(
            locale
                .sections
                .hand_cards
                .replace("{cards}", &cards.join(" ")),
        );
    }

    lines.join("\n")
}
