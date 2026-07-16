use std::collections::HashSet;

use serde_json::{Map, Value, json};

use crate::autoplay::control::AutoPlaySession;
use crate::locales::Locale;
use crate::state::NormalizedState;

use super::map::map_context;
use super::projection::{
    card_value, card_values, deck_value, indexed_card_value, monster_value, pile_value,
    potion_value, relic_value, with_index,
};

pub(super) fn add_screen_context(
    scenario: &mut Map<String, Value>,
    kind: &str,
    session: &AutoPlaySession,
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
) {
    match kind {
        "combat" => insert(scenario, "combat", combat_context(state, locale)),
        "card_reward" | "boss_card_reward" => {
            insert(
                scenario,
                "deck",
                deck_value(&state.master_cards, &state.deck_names, locale),
            );
            insert(
                scenario,
                "card_reward",
                card_reward_context(state, locale, kind == "boss_card_reward"),
            );
        }
        "boss_relic" => {
            insert(
                scenario,
                "deck",
                deck_value(&state.master_cards, &state.deck_names, locale),
            );
            insert(
                scenario,
                "boss_relic_reward",
                boss_relic_context(state, locale),
            );
        }
        "rest" => {
            insert(
                scenario,
                "deck",
                deck_value(&state.master_cards, &state.deck_names, locale),
            );
            insert(scenario, "rest", rest_context(state, locale));
        }
        "event" => insert(scenario, "event", event_context(state)),
        "shop" => {
            insert(
                scenario,
                "deck",
                deck_value(&state.master_cards, &state.deck_names, locale),
            );
            insert(scenario, "shop", shop_context(state, locale, shop_visited));
        }
        "map_crossroad" | "map_suggestion" => {
            insert(scenario, "map", map_context(state, locale, shop_visited));
        }
        "hand_select" => insert(scenario, "hand_select", hand_select_context(state, locale)),
        "grid" => {
            insert(
                scenario,
                "deck",
                deck_value(&state.master_cards, &state.deck_names, locale),
            );
            insert(scenario, "grid", grid_context(session, state, locale));
        }
        "combat_reward" => insert(
            scenario,
            "combat_reward",
            json!({
                "empty_potion_slots": state.empty_potion_slots,
                "card_reward_already_skipped": session.skipped_combat_reward_card,
                "potion_reward_already_skipped": session.skipped_combat_reward_potion,
            }),
        ),
        _ => insert(scenario, "generic", generic_context(state, locale)),
    }
}

fn combat_context(state: &NormalizedState, locale: &Locale) -> Value {
    let room_type = state.room_type.as_ref().map(|room| room.as_str());
    json!({
        "turn": state.turn_number,
        "encounter_type": room_type,
        "is_elite": room_type == Some("MonsterRoomElite"),
        "is_boss": room_type == Some("MonsterRoomBoss"),
        "hand": card_values(&state.hand, "hand_index", locale),
        "monsters": state.monsters.iter().map(monster_value).collect::<Vec<_>>(),
        "piles": {
            "draw": pile_value(&state.draw_pile, locale),
            "discard": pile_value(&state.discard_pile, locale),
            "exhaust": pile_value(&state.exhaust_cards, locale),
        },
    })
}

fn card_reward_context(
    state: &NormalizedState,
    locale: &Locale,
    next_act_full_heal: bool,
) -> Value {
    json!({
        "choices": state.card_reward_choices.iter().enumerate().map(|(index, card)| {
            indexed_card_value(card, "choice_index", index, locale)
        }).collect::<Vec<_>>(),
        "skip_available": state.skip_available,
        "next_act_full_heal": next_act_full_heal,
    })
}

fn boss_relic_context(state: &NormalizedState, locale: &Locale) -> Value {
    json!({
        "choices": state.boss_relic_choices.iter().enumerate().map(|(index, relic)| {
            with_index(relic_value(relic, locale), "choice_index", index)
        }).collect::<Vec<_>>(),
        "next_act_full_heal": matches!(state.floor, Some(17 | 34)),
    })
}

fn rest_context(state: &NormalizedState, locale: &Locale) -> Value {
    let mut seen = HashSet::new();
    let mut upgrade_targets: Vec<_> = state
        .master_cards
        .iter()
        .filter(|card| !card.upgraded && !matches!(card.card_type.as_str(), "CURSE" | "STATUS"))
        .filter(|card| seen.insert(card.id.as_str()))
        .collect();
    upgrade_targets.sort_by_key(|card| card.name.as_str());
    let hp_ratio = state
        .current_hp
        .zip(state.max_hp)
        .filter(|(_, max_hp)| *max_hp > 0)
        .map(|(current_hp, max_hp)| current_hp as f64 / max_hp as f64);
    let health_guidance = hp_ratio.map(|ratio| {
        if ratio < 0.3 {
            "rest_recommended"
        } else if ratio > 0.7 {
            "upgrade_or_relic_action_recommended"
        } else {
            "neutral"
        }
    });
    json!({
        "options": state.rest_options,
        "hp_ratio": hp_ratio,
        "health_guidance": health_guidance,
        "upgrade_targets": upgrade_targets.into_iter().take(10)
            .map(|card| card_value(card, locale))
            .collect::<Vec<_>>(),
    })
}

fn event_context(state: &NormalizedState) -> Value {
    json!({
        "id": state.event_id,
        "name": state.event_name,
        "body": state.event_body,
        "choices": state.event_choices,
    })
}

fn shop_context(state: &NormalizedState, locale: &Locale, shop_visited: bool) -> Value {
    json!({
        "cards": card_values(&state.shop_cards, "shop_index", locale),
        "relics": state.shop_relics.iter().enumerate().map(|(index, relic)| {
            with_index(relic_value(relic, locale), "shop_index", index)
        }).collect::<Vec<_>>(),
        "potions": state.shop_potions.iter().enumerate().map(|(index, potion)| {
            with_index(potion_value(potion, locale), "shop_index", index)
        }).collect::<Vec<_>>(),
        "purge_available": state.purge_available,
        "purge_cost": state.purge_cost,
        "visited": shop_visited,
    })
}

fn hand_select_context(state: &NormalizedState, locale: &Locale) -> Value {
    json!({
        "current_action": state.current_action,
        "purpose": selection_purpose(state.current_action.as_deref()),
        "max_cards": state.hand_select_max_cards,
        "can_pick_zero": state.hand_select_can_pick_zero,
        "card_in_play": state.card_in_play.as_ref().map(|card| card_value(card, locale)),
        "selected_cards": card_values(&state.hand_select_selected, "selected_index", locale),
        "available_cards": card_values(&state.hand, "choice_index", locale),
    })
}

fn grid_context(session: &AutoPlaySession, state: &NormalizedState, locale: &Locale) -> Value {
    let cards = if state.grid_cards.is_empty() {
        &state.hand
    } else {
        &state.grid_cards
    };
    json!({
        "purpose": grid_purpose(session, state),
        "num_cards": state.grid_num_cards,
        "triggered_by_relic": session.pending_boss_relic_grid,
        "selected_cards": card_values(&state.grid_selected_cards, "selected_index", locale),
        "available_cards": card_values(cards, "choice_index", locale),
    })
}

fn generic_context(state: &NormalizedState, locale: &Locale) -> Value {
    json!({
        "hand": card_values(&state.hand, "hand_index", locale),
        "monsters": state.monsters.iter().map(monster_value).collect::<Vec<_>>(),
    })
}

fn selection_purpose(action: Option<&str>) -> Option<&str> {
    match action {
        Some("ExhaustAction") => Some("exhaust"),
        Some("DiscardAction") => Some("discard"),
        Some("PutOnDeckAction") => Some("put_on_draw_pile"),
        Some(_) | None => None,
    }
}

fn grid_purpose<'a>(session: &'a AutoPlaySession, state: &NormalizedState) -> Option<&'a str> {
    if state.grid_for_upgrade {
        Some("upgrade")
    } else if state.grid_for_transform {
        Some("transform")
    } else if state.grid_for_purge {
        Some("purge")
    } else if session.pending_boss_relic_grid.is_some() {
        Some("relic")
    } else {
        None
    }
}

fn insert(scenario: &mut Map<String, Value>, key: &str, value: impl Into<Value>) {
    scenario.insert(key.to_string(), value.into());
}
