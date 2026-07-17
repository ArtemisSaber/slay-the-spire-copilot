use serde_json::{Value, json};

use crate::locales::Locale;
use crate::state::NormalizedState;

use super::super::projection::indexed_card_value;

pub(super) fn combat_card_choice_context(state: &NormalizedState, locale: &Locale) -> Value {
    json!({
        "current_action": state.current_action,
        "scope": "current_combat",
        "permanent_deck_change": false,
        "choices": state.card_reward_choices.iter().enumerate().map(|(index, card)| {
            indexed_card_value(card, "choice_index", index, locale)
        }).collect::<Vec<_>>(),
        "skip_available": state.skip_available,
    })
}
