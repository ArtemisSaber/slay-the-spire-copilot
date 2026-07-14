use crate::locales::Locale;
use crate::state::NormalizedState;

use super::combat_prompt::build_combat;
use super::map_prompt::{build_map_crossroad, build_map_suggestion};
use super::rewards::{build_boss_relic, build_card_reward, build_rest};
use super::selection::{build_grid_select, build_hand_select};
use super::shop_event::{build_event_choice, build_generic, build_shop};

pub fn build_prompt(state: &NormalizedState, locale: &Locale, shop_visited: bool) -> String {
    match state
        .screen_type
        .as_ref()
        .map(|screen_type| screen_type.as_str())
    {
        Some("CARD_REWARD") => build_card_reward(state, locale),
        Some("BOSS_REWARD") => build_boss_relic(state, locale),
        Some("REST") => build_rest(state, locale),
        Some("EVENT") => build_event_choice(state, locale),
        Some("SHOP_SCREEN") => build_shop(state, locale),
        Some("MAP") if state.map_first_node_chosen == Some(true) => {
            build_map_crossroad(state, locale, shop_visited)
        }
        Some("MAP") => build_map_suggestion(state, locale),
        Some("HAND_SELECT") => build_hand_select(state, locale),
        Some("GRID") => build_grid_select(state, locale),
        _ if !state.monsters.is_empty() => build_combat(state, locale),
        _ => build_generic(state, locale),
    }
}
