use serde_json::Value;

use super::super::NormalizedState;
use super::helpers::{
    insert_bool_if_true, insert_opt_bool, insert_opt_i64, insert_opt_str, sorted_id_only_array,
};

pub(super) fn insert_screen_fields(
    map: &mut serde_json::Map<String, Value>,
    state: &NormalizedState,
) {
    let mut sorted_shop_cards = state.shop_cards.clone();
    sorted_shop_cards.sort_by(|left, right| left.id.cmp(&right.id));
    map.insert(
        "shop_cards".to_string(),
        Value::Array(
            sorted_shop_cards
                .into_iter()
                .map(|card| {
                    let mut card_map = serde_json::Map::new();
                    card_map.insert("id".to_string(), Value::String(card.id.clone()));
                    card_map.insert("cost".to_string(), Value::Number(card.cost.into()));
                    card_map.insert(
                        "price".to_string(),
                        Value::Number(card.price.unwrap_or(0).into()),
                    );
                    Value::Object(card_map)
                })
                .collect(),
        ),
    );

    let mut sorted_shop_relics = state.shop_relics.clone();
    sorted_shop_relics.sort_by(|left, right| left.name.cmp(&right.name));
    map.insert(
        "shop_relics".to_string(),
        Value::Array(
            sorted_shop_relics
                .into_iter()
                .map(|relic| {
                    let mut relic_map = serde_json::Map::new();
                    relic_map.insert("name".to_string(), Value::String(relic.name.clone()));
                    relic_map.insert(
                        "price".to_string(),
                        Value::Number(relic.price.unwrap_or(0).into()),
                    );
                    Value::Object(relic_map)
                })
                .collect(),
        ),
    );

    if state.purge_available {
        map.insert("purge_available".to_string(), Value::Bool(true));
        insert_opt_i64(map, "purge_cost", state.purge_cost);
    }

    insert_opt_bool(map, "map_first_node_chosen", state.map_first_node_chosen);
    insert_opt_i64(map, "map_current_x", state.map_current_x);
    insert_opt_i64(map, "map_current_y", state.map_current_y);
    insert_opt_i64(map, "hand_select_max_cards", state.hand_select_max_cards);
    insert_bool_if_true(
        map,
        "hand_select_can_pick_zero",
        state.hand_select_can_pick_zero,
    );
    if !state.hand_select_selected.is_empty() {
        map.insert(
            "hand_select_selected".to_string(),
            sorted_id_only_array(&state.hand_select_selected),
        );
    }
    insert_opt_str(map, "current_action", &state.current_action);
    if let Some(card) = &state.card_in_play {
        let mut card_map = serde_json::Map::new();
        card_map.insert("id".to_string(), Value::String(card.id.clone()));
        card_map.insert("cost".to_string(), Value::Number(card.cost.into()));
        card_map.insert("type".to_string(), Value::String(card.card_type.clone()));
        card_map.insert("upgraded".to_string(), Value::Bool(card.upgraded));
        map.insert("card_in_play".to_string(), Value::Object(card_map));
    }
    if !state.grid_cards.is_empty() {
        map.insert(
            "grid_cards".to_string(),
            sorted_id_only_array(&state.grid_cards),
        );
    }
    if !state.grid_selected_cards.is_empty() {
        map.insert(
            "grid_selected_cards".to_string(),
            sorted_id_only_array(&state.grid_selected_cards),
        );
    }
    insert_bool_if_true(map, "grid_for_upgrade", state.grid_for_upgrade);
    insert_bool_if_true(map, "grid_for_transform", state.grid_for_transform);
    insert_bool_if_true(map, "grid_for_purge", state.grid_for_purge);
    insert_opt_i64(map, "grid_num_cards", state.grid_num_cards);
    if state.empty_potion_slots > 0 {
        map.insert(
            "empty_potion_slots".to_string(),
            Value::Number(state.empty_potion_slots.into()),
        );
    }
}
