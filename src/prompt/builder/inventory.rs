use crate::locales::Locale;
use crate::relic_counters::rewrite_relic_description;
use crate::state::NormalizedState;

use super::cards::clean_description;

pub(crate) fn build_relics_potions_section(state: &NormalizedState, locale: &Locale) -> String {
    let mut lines = Vec::new();

    if !state.relics.is_empty() {
        lines.push(locale.sections.relics.clone());
        for relic in &state.relics {
            let name = match relic.counter {
                Some(counter) => format!("{} ({})", relic.name, counter),
                None => relic.name.clone(),
            };
            let description = match relic.counter {
                Some(counter) => {
                    rewrite_relic_description(&relic.id, counter, &relic.description, locale)
                }
                None => relic.description.clone(),
            };
            lines.push(format!(
                "{}：{}",
                name,
                clean_description(&description, locale)
            ));
        }
        lines.push(String::new());
    }

    if !state.potions.is_empty() {
        lines.push(locale.sections.potions.clone());
        for potion in &state.potions {
            lines.push(format!(
                "{}：{}",
                potion.name,
                clean_description(&potion.description, locale)
            ));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}
