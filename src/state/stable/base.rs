use serde_json::Value;

use super::super::NormalizedState;
use super::helpers::{
    insert_opt_i64, insert_opt_str, sorted_id_only_array, sorted_relic_name_desc_array,
    sorted_string_array,
};

pub(super) fn insert_base_fields(
    map: &mut serde_json::Map<String, Value>,
    state: &NormalizedState,
) {
    insert_opt_str(map, "screen_type", &state.screen_type);
    insert_opt_str(map, "room_type", &state.room_type);
    insert_opt_str(map, "room_phase", &state.room_phase);
    insert_opt_str(map, "character", &state.character);
    insert_opt_i64(map, "floor", state.floor);
    insert_opt_i64(map, "current_hp", state.current_hp);
    insert_opt_i64(map, "max_hp", state.max_hp);
    insert_opt_i64(map, "gold", state.gold);
    insert_opt_i64(map, "energy", state.energy);
    insert_opt_i64(map, "block", state.block);
    map.insert(
        "incoming_damage".to_string(),
        Value::Number(state.incoming_damage.into()),
    );

    let mut sorted_powers = state.powers.clone();
    sorted_powers.sort_by(|left, right| left.name.cmp(&right.name));
    map.insert(
        "powers".to_string(),
        Value::Array(
            sorted_powers
                .iter()
                .map(|power| {
                    let mut power_map = serde_json::Map::new();
                    power_map.insert("name".to_string(), Value::String(power.name.clone()));
                    power_map.insert("amount".to_string(), Value::Number(power.amount.into()));
                    Value::Object(power_map)
                })
                .collect(),
        ),
    );

    let mut sorted_hand = state.hand.clone();
    sorted_hand.sort_by(|left, right| left.id.cmp(&right.id));
    map.insert(
        "hand".to_string(),
        Value::Array(
            sorted_hand
                .iter()
                .map(|card| {
                    let mut card_map = serde_json::Map::new();
                    card_map.insert("id".to_string(), Value::String(card.id.clone()));
                    card_map.insert("cost".to_string(), Value::Number(card.cost.into()));
                    card_map.insert("type".to_string(), Value::String(card.card_type.clone()));
                    card_map.insert("upgraded".to_string(), Value::Bool(card.upgraded));
                    Value::Object(card_map)
                })
                .collect(),
        ),
    );

    let mut sorted_monsters = state.monsters.clone();
    sorted_monsters.sort_by(|left, right| left.name.cmp(&right.name));
    map.insert(
        "monsters".to_string(),
        Value::Array(
            sorted_monsters
                .iter()
                .map(|monster| {
                    let mut monster_map = serde_json::Map::new();
                    monster_map.insert("name".to_string(), Value::String(monster.name.clone()));
                    insert_opt_i64(&mut monster_map, "current_hp", monster.current_hp);
                    insert_opt_i64(&mut monster_map, "max_hp", monster.max_hp);
                    insert_opt_str(&mut monster_map, "intent", &monster.intent);
                    insert_opt_i64(&mut monster_map, "damage", monster.damage);
                    Value::Object(monster_map)
                })
                .collect(),
        ),
    );

    map.insert(
        "card_reward_choices".to_string(),
        sorted_id_only_array(&state.card_reward_choices),
    );
    map.insert(
        "boss_relic_choices".to_string(),
        sorted_relic_name_desc_array(&state.boss_relic_choices),
    );
    insert_opt_str(map, "event_name", &state.event_name);
    insert_opt_str(map, "event_id", &state.event_id);
    insert_opt_str(map, "event_body", &state.event_body);
    map.insert(
        "event_choices".to_string(),
        sorted_string_array(&state.event_choices),
    );
    map.insert(
        "relics".to_string(),
        sorted_relic_name_desc_array(&state.relics),
    );

    let mut sorted_potions = state.potions.clone();
    sorted_potions.sort_by(|left, right| left.name.cmp(&right.name));
    map.insert(
        "potions".to_string(),
        Value::Array(
            sorted_potions
                .into_iter()
                .map(|potion| {
                    let mut potion_map = serde_json::Map::new();
                    potion_map.insert("name".to_string(), Value::String(potion.name.clone()));
                    potion_map.insert(
                        "description".to_string(),
                        Value::String(potion.description.clone()),
                    );
                    Value::Object(potion_map)
                })
                .collect(),
        ),
    );
    map.insert(
        "deck_names".to_string(),
        sorted_string_array(&state.deck_names),
    );
    map.insert(
        "rest_options".to_string(),
        sorted_string_array(&state.rest_options),
    );
    map.insert(
        "skip_available".to_string(),
        Value::Bool(state.skip_available),
    );
}
