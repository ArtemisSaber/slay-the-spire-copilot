use std::collections::HashSet;

use crate::locales::Locale;
use crate::state::{CardInfo, NormalizedState};

use super::cards::{clean_description, format_card, format_deck_section};
use super::inventory::build_relics_potions_section;
use super::status::status_line;

pub(crate) fn build_card_reward(state: &NormalizedState, locale: &Locale) -> String {
    let mode = if state.is_boss_card_reward() {
        "[mode: boss_card_reward]\n"
    } else {
        "[mode: card_reward]\n"
    };

    let mut lines: Vec<String> = vec![
        mode.to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    if state.is_boss_card_reward() {
        lines.push(locale.warnings.boss_card_hp_note.clone());
        lines.push(String::new());
    }

    lines.push(format_deck_section(&state.master_cards, locale));

    if !state.card_reward_choices.is_empty() {
        lines.push(locale.sections.card_reward.clone());
        for (index, card) in state.card_reward_choices.iter().enumerate() {
            let label = (b'A' + index as u8) as char;
            lines.push(format!("{label}. {}", format_card(card, locale)));
        }
        if state.skip_available {
            lines.push(locale.warnings.skip.clone());
        }
    }

    lines.join("\n")
}

pub(crate) fn build_rest(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: rest]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
        String::new(),
        build_relics_potions_section(state, locale),
    ];

    let mut seen_upgradeable = HashSet::new();
    let mut upgradeable: Vec<&CardInfo> = state
        .master_cards
        .iter()
        .filter(|card| !card.upgraded && card.card_type != "CURSE" && card.card_type != "STATUS")
        .filter(|card| seen_upgradeable.insert(card.id.as_str()))
        .take(10)
        .collect();
    upgradeable.sort_by_key(|card| card.name.as_str());

    if !upgradeable.is_empty() {
        let names: Vec<String> = upgradeable
            .into_iter()
            .map(|card| format_card(card, locale))
            .collect();
        lines.push(locale.sections.upgrade_targets.clone());
        lines.extend(names);
        lines.push(String::new());
    }

    if !state.rest_options.is_empty() {
        lines.push(locale.sections.options.clone());
        let options: Vec<String> = state
            .rest_options
            .iter()
            .map(|option| {
                (match option.as_str() {
                    "rest" => &locale.i18n.rest_rest,
                    "smith" => &locale.i18n.rest_smith,
                    "toke" => &locale.i18n.rest_toke,
                    "dig" => &locale.i18n.rest_dig,
                    "lift" => &locale.i18n.rest_lift,
                    "recall" => &locale.i18n.rest_recall,
                    "girya" => &locale.i18n.rest_girya,
                    _ => option.as_str(),
                })
                .to_string()
            })
            .collect();
        lines.push(options.join("  "));
        lines.push(String::new());
    }

    if let (Some(current_hp), Some(max_hp)) = (state.current_hp, state.max_hp)
        && max_hp > 0
    {
        let percent = current_hp as f64 / max_hp as f64;
        if percent < 0.3 {
            lines.push(locale.warnings.low_hp_rest.clone());
        } else if percent > 0.7 {
            lines.push(locale.warnings.high_hp_smith.clone());
        }
    }

    lines.join("\n")
}

pub(crate) fn build_boss_relic(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines: Vec<String> = vec![
        "[mode: boss_relic]\n".to_string(),
        locale.sections.current_state.clone(),
        status_line(state, locale),
    ];

    let is_act_end = matches!(state.floor, Some(17) | Some(34));
    if is_act_end {
        lines.push(locale.warnings.boss_relic_hp_note.clone());
    }

    lines.push(String::new());
    lines.push(build_relics_potions_section(state, locale));
    lines.push(format_deck_section(&state.master_cards, locale));

    if !state.boss_relic_choices.is_empty() {
        lines.push(locale.sections.boss_relic.clone());
        for (index, relic) in state.boss_relic_choices.iter().enumerate() {
            let label = (b'A' + index as u8) as char;
            lines.push(format!(
                "{label}. {} — {}",
                relic.name,
                clean_description(&relic.description, locale)
            ));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}
