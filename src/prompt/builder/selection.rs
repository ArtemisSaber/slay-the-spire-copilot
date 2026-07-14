use crate::locales::Locale;
use crate::state::NormalizedState;

use super::cards::format_card;
use super::inventory::build_relics_potions_section;
use super::status::status_line;

pub(crate) fn build_hand_select(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: hand_select]\n".to_string(),
        locale.sections.hand_select.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    if let Some(action) = &state.current_action {
        let action_description = match action.as_str() {
            "ExhaustAction" => "Exhaust a card".to_string(),
            "DiscardAction" => "Discard a card".to_string(),
            "PutOnDeckAction" => "Put a card on top of your draw pile".to_string(),
            _ => action.to_string(),
        };
        if let Some(card) = &state.card_in_play {
            lines.push(format!("Card playing: {}", format_card(card, locale)));
        }
        lines.push(format!("Purpose: {action_description}"));
    }
    if let Some(max_cards) = state.hand_select_max_cards {
        lines.push(format!(
            "Max: {max_cards}  Can skip: {}",
            if state.hand_select_can_pick_zero {
                "yes"
            } else {
                "no"
            }
        ));
    }

    if !state.hand_select_selected.is_empty() {
        let selected: Vec<String> = state
            .hand_select_selected
            .iter()
            .map(|card| card.name.clone())
            .collect();
        lines.push(format!(
            "{} [{}]",
            locale.sections.selected_cards,
            selected.join(", ")
        ));
    }

    lines.push(String::new());
    lines.push(locale.sections.hand_select_available.clone());
    for (index, card) in state.hand.iter().enumerate() {
        lines.push(format!("  {index}. {}", format_card(card, locale)));
    }

    lines.join("\n")
}

pub(crate) fn build_grid_select(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: grid_select]\n".to_string(),
        locale.sections.grid_select.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    let purpose = if state.grid_for_upgrade {
        locale.i18n.grid_upgrade.as_str()
    } else if state.grid_for_transform {
        locale.i18n.grid_transform.as_str()
    } else if state.grid_for_purge {
        locale.i18n.grid_purge.as_str()
    } else {
        locale.i18n.grid_other.as_str()
    };
    lines.push(purpose.to_string());
    if let Some(num_cards) = state.grid_num_cards {
        lines.push(format!("Select {} card(s).", num_cards));
    }
    lines.push(String::new());

    lines.push(locale.sections.hand_select_available.clone());
    let cards = if !state.grid_cards.is_empty() {
        &state.grid_cards
    } else {
        &state.hand
    };
    for (index, card) in cards.iter().enumerate() {
        lines.push(format!("  {index}. {}", format_card(card, locale)));
    }

    lines.join("\n")
}
