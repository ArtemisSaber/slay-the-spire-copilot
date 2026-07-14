use crate::locales::Locale;
use crate::state::{DangerLevel, NormalizedState};

pub(crate) fn danger_prefix(state: &NormalizedState, locale: &Locale) -> String {
    let mut reasons: Vec<&str> = Vec::new();

    if state.danger.incoming_lethal {
        reasons.push(&locale.danger.fatal_damage);
    }
    if state.danger.hp_critical {
        reasons.push(&locale.danger.hp_critical);
    }
    if state.danger.wrath_stance && state.danger.any_monster_attacking {
        reasons.push(&locale.danger.wrath_stance);
    }
    if state.danger.no_block_against_hit {
        reasons.push(&locale.danger.no_block);
    }

    match state.danger.level {
        DangerLevel::Danger => {
            if reasons.is_empty() {
                locale.danger.danger_prefix.clone()
            } else {
                locale.danger.danger_with_reasons.replace(
                    "{reasons}",
                    &reasons.join(&locale.danger.danger_reason_separator),
                )
            }
        }
        DangerLevel::Caution => locale.danger.caution.clone(),
        DangerLevel::Safe => locale.danger.safe.clone(),
    }
}

pub(crate) fn combat_profile_line(state: &NormalizedState, locale: &Locale) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(character) = &state.character {
        let class_name = match character.as_str() {
            "IRONCLAD" => &locale.i18n.class_ironclad,
            "THE_SILENT" => &locale.i18n.class_silent,
            "DEFECT" => &locale.i18n.class_defect,
            "WATCHER" => &locale.i18n.class_watcher,
            _ => character.as_str(),
        };
        parts.push(locale.status.character.replace("{class}", class_name));
    }
    if let Some(floor) = state.floor {
        parts.push(locale.status.floor.replace("{floor}", &floor.to_string()));
    }
    if let Some(max_hp) = state.max_hp {
        parts.push(locale.status.max_hp.replace("{max}", &max_hp.to_string()));
    }
    if let Some(gold) = state.gold {
        parts.push(locale.status.gold.replace("{gold}", &gold.to_string()));
    }
    if !state.relics.is_empty() {
        let names: Vec<String> = state
            .relics
            .iter()
            .map(|relic| match relic.counter {
                Some(counter) => format!("{} ({})", relic.name, counter),
                None => relic.name.clone(),
            })
            .collect();
        parts.push(locale.status.relics.replace("{list}", &names.join(" ")));
    }

    parts.join("  ")
}

pub(crate) fn turn_status_line(state: &NormalizedState, locale: &Locale) -> String {
    let mut parts: Vec<String> = Vec::new();

    parts.push(danger_prefix(state, locale));

    if let (Some(current_hp), Some(max_hp)) = (state.current_hp, state.max_hp) {
        let percent = if max_hp > 0 {
            (current_hp as f64 / max_hp as f64 * 100.0) as i64
        } else {
            0
        };
        parts.push(
            locale
                .status
                .hp
                .replace("{cur}", &current_hp.to_string())
                .replace("{max}", &max_hp.to_string())
                .replace("{pct}", &percent.to_string()),
        );
    }
    if let Some(block) = state.block {
        parts.push(locale.status.block.replace("{block}", &block.to_string()));
    }
    if let Some(energy) = state.energy {
        parts.push(
            locale
                .status
                .energy
                .replace("{energy}", &energy.to_string()),
        );
    }

    if !state.powers.is_empty() {
        let powers: Vec<String> = state
            .powers
            .iter()
            .map(|power| format!("{}({})", power.name, power.amount))
            .collect();
        parts.push(locale.status.powers.replace("{list}", &powers.join(" ")));
    }

    if !state.potions.is_empty() {
        let names: Vec<&str> = state
            .potions
            .iter()
            .map(|potion| potion.name.as_str())
            .collect();
        parts.push(locale.status.potions.replace("{list}", &names.join(" ")));
    }

    if state.incoming_damage > 0 {
        let warning =
            if state.block.unwrap_or(0) > 0 && state.incoming_damage > state.block.unwrap_or(0) {
                locale.status.need_block.as_str()
            } else {
                ""
            };
        parts.push(
            locale
                .status
                .damage_total
                .replace("{dmg}", &state.incoming_damage.to_string())
                .replace("{warn}", warning),
        );
    }

    parts.join("  ")
}

pub(crate) fn status_line(state: &NormalizedState, locale: &Locale) -> String {
    let mut parts: Vec<String> = Vec::new();

    parts.push(danger_prefix(state, locale));

    if let Some(character) = &state.character {
        let class_name = match character.as_str() {
            "IRONCLAD" => &locale.i18n.class_ironclad,
            "THE_SILENT" => &locale.i18n.class_silent,
            "DEFECT" => &locale.i18n.class_defect,
            "WATCHER" => &locale.i18n.class_watcher,
            _ => character.as_str(),
        };
        parts.push(locale.status.character.replace("{class}", class_name));
    }
    if let Some(floor) = state.floor {
        parts.push(locale.status.floor.replace("{floor}", &floor.to_string()));
    }
    if let (Some(current_hp), Some(max_hp)) = (state.current_hp, state.max_hp) {
        let percent = if max_hp > 0 {
            (current_hp as f64 / max_hp as f64 * 100.0) as i64
        } else {
            0
        };
        parts.push(
            locale
                .status
                .hp
                .replace("{cur}", &current_hp.to_string())
                .replace("{max}", &max_hp.to_string())
                .replace("{pct}", &percent.to_string()),
        );
    }
    if let Some(block) = state.block {
        parts.push(locale.status.block.replace("{block}", &block.to_string()));
    }
    if let Some(energy) = state.energy {
        parts.push(
            locale
                .status
                .energy
                .replace("{energy}", &energy.to_string()),
        );
    }
    if let Some(gold) = state.gold {
        parts.push(locale.status.gold.replace("{gold}", &gold.to_string()));
    }

    if !state.powers.is_empty() {
        let powers: Vec<String> = state
            .powers
            .iter()
            .map(|power| format!("{}({})", power.name, power.amount))
            .collect();
        parts.push(locale.status.powers.replace("{list}", &powers.join(" ")));
    }

    if !state.relics.is_empty() {
        let names: Vec<String> = state
            .relics
            .iter()
            .map(|relic| match relic.counter {
                Some(counter) => format!("{} ({})", relic.name, counter),
                None => relic.name.clone(),
            })
            .collect();
        parts.push(locale.status.relics.replace("{list}", &names.join(" ")));
    }
    if !state.potions.is_empty() {
        let names: Vec<&str> = state
            .potions
            .iter()
            .map(|potion| potion.name.as_str())
            .collect();
        parts.push(locale.status.potions.replace("{list}", &names.join(" ")));
    }

    if state.incoming_damage > 0 {
        let warning =
            if state.block.unwrap_or(0) > 0 && state.incoming_damage > state.block.unwrap_or(0) {
                locale.status.need_block.as_str()
            } else {
                ""
            };
        parts.push(
            locale
                .status
                .damage_total
                .replace("{dmg}", &state.incoming_damage.to_string())
                .replace("{warn}", warning),
        );
    }

    parts.join("  ")
}
