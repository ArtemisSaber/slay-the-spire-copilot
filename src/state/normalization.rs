use serde_json::Value;

use crate::locales::Locale;

use super::combat::{detect_stance_from_powers, extract_monsters, extract_orbs};
use super::event::extract_event_fields;
use super::parse::{
    extract_card_names, extract_cards, extract_potion_infos, extract_powers, extract_relic_infos,
    first_array,
};
use super::screen_data::{
    extract_empty_potion_slots, extract_grid_fields, extract_hand_select_fields, extract_map_nodes,
    extract_shop_fields,
};
use super::{CardInfo, DangerFlags, NormalizedState, RoomType, ScreenType};

fn monster_incoming_damage(monster: &super::MonsterInfo) -> i64 {
    if monster.intent.as_deref() == Some("NONE") {
        return 0;
    }

    let Some(damage_per_hit) = monster.damage.filter(|damage| *damage > 0) else {
        return 0;
    };
    let hit_count = monster.hits.unwrap_or(1).max(0);

    damage_per_hit.saturating_mul(hit_count)
}

impl NormalizedState {
    pub fn from_raw(raw: &Value, locale: &Locale) -> Self {
        let game_state = raw.get("game_state");
        let screen_type = game_state
            .and_then(|state| state.get("screen_type"))
            .and_then(|value| serde_json::from_value::<ScreenType>(value.clone()).ok());
        let room_type = game_state
            .and_then(|state| state.get("room_type"))
            .and_then(|value| serde_json::from_value::<RoomType>(value.clone()).ok());
        let room_phase = game_state
            .and_then(|state| state.get("room_phase"))
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let character = game_state
            .and_then(|state| state.get("class"))
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let seed = game_state
            .and_then(|state| state.get("seed"))
            .and_then(|value| value.as_i64());
        let ascension_level = game_state
            .and_then(|state| state.get("ascension_level"))
            .and_then(|value| value.as_i64());
        let floor = game_state
            .and_then(|state| state.get("floor"))
            .and_then(|value| value.as_i64());
        let current_hp = game_state
            .and_then(|state| state.get("current_hp"))
            .and_then(|value| value.as_i64());
        let max_hp = game_state
            .and_then(|state| state.get("max_hp"))
            .and_then(|value| value.as_i64());
        let gold = game_state
            .and_then(|state| state.get("gold"))
            .and_then(|value| value.as_i64());

        let combat = game_state.and_then(|state| state.get("combat_state"));
        let player = combat.and_then(|state| state.get("player"));
        let energy = player
            .and_then(|state| state.get("energy"))
            .and_then(|value| value.as_i64());
        let block = player
            .and_then(|state| state.get("block"))
            .and_then(|value| value.as_i64());
        let powers = player
            .and_then(|state| state.get("powers"))
            .and_then(|value| value.as_array())
            .map(|powers| extract_powers(powers))
            .unwrap_or_default();
        let turn_number = combat
            .and_then(|state| state.get("turn"))
            .and_then(|value| value.as_i64());
        let orbs = extract_orbs(player);
        let stance = detect_stance_from_powers(&powers);

        let hand = combat
            .and_then(|state| state.get("hand"))
            .and_then(|value| value.as_array())
            .map(|cards| extract_cards(cards))
            .unwrap_or_default();
        let hand_cards = hand.clone();
        let draw_pile = combat
            .and_then(|state| state.get("draw_pile"))
            .and_then(|value| value.as_array())
            .map(|cards| extract_cards(cards))
            .unwrap_or_default();
        let discard_pile = combat
            .and_then(|state| state.get("discard_pile"))
            .and_then(|value| value.as_array())
            .map(|cards| extract_cards(cards))
            .unwrap_or_default();
        let exhaust_cards = combat
            .and_then(|state| state.get("exhaust_pile"))
            .and_then(|value| value.as_array())
            .map(|cards| extract_cards(cards))
            .unwrap_or_default();
        let total_hand_atk = hand_cards
            .iter()
            .filter(|card| card.card_type == "ATTACK")
            .map(|card| card.cost.min(1) * 6)
            .sum();
        let monsters = extract_monsters(combat, total_hand_atk);
        let incoming_damage = monsters
            .iter()
            .map(monster_incoming_damage)
            .fold(0_i64, i64::saturating_add);
        let danger = DangerFlags::compute(
            current_hp,
            max_hp,
            block,
            incoming_damage,
            &monsters,
            &powers,
        );

        let screen_state = game_state.and_then(|state| state.get("screen_state"));
        let map_first_node_chosen = screen_state
            .and_then(|state| state.get("first_node_chosen"))
            .and_then(|value| value.as_bool());
        let map_current_x = screen_state
            .and_then(|state| state.get("current_node"))
            .and_then(|node| node.get("x"))
            .and_then(|value| value.as_i64());
        let map_current_y = screen_state
            .and_then(|state| state.get("current_node"))
            .and_then(|node| node.get("y"))
            .and_then(|value| value.as_i64());
        let skip_available = screen_state
            .and_then(|state| state.get("skip_available"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let card_reward_choices = screen_state
            .and_then(|state| state.get("cards"))
            .and_then(|value| value.as_array())
            .map(|cards| extract_cards(cards))
            .unwrap_or_default();
        let boss_relic_choices = screen_state
            .and_then(|state| first_array(state, &["relics", "boss_relics", "relic_options"]))
            .map(|relics| extract_relic_infos(relics))
            .unwrap_or_default();
        let event = extract_event_fields(screen_state, game_state, locale);
        let rest_options = screen_state
            .and_then(|state| state.get("rest_options"))
            .and_then(|value| value.as_array())
            .map(|options| {
                options
                    .iter()
                    .filter_map(|option| option.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let shop = extract_shop_fields(screen_state);
        let master_cards = game_state
            .and_then(|state| state.get("deck"))
            .and_then(|value| value.as_array())
            .map(|cards| extract_cards(cards))
            .unwrap_or_default();
        let map_nodes = extract_map_nodes(game_state);
        let mut relics = game_state
            .and_then(|state| state.get("relics"))
            .and_then(|value| value.as_array())
            .map(|relics| extract_relic_infos(relics))
            .unwrap_or_default();
        let mut potions = game_state
            .and_then(|state| state.get("potions"))
            .and_then(|value| value.as_array())
            .map(|potions| extract_potion_infos(potions))
            .unwrap_or_default();
        let empty_potion_slots = extract_empty_potion_slots(game_state);
        let hand_select = extract_hand_select_fields(screen_state);
        let current_action = game_state
            .and_then(|state| state.get("current_action"))
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let card_in_play = combat
            .and_then(|state| state.get("card_in_play"))
            .map(CardInfo::from_json);
        let grid = extract_grid_fields(screen_state);
        let mut deck_names = game_state
            .and_then(|state| state.get("deck"))
            .and_then(|value| value.as_array())
            .map(|cards| extract_card_names(cards))
            .unwrap_or_default();

        relics.sort_by(|left, right| left.name.cmp(&right.name));
        potions.sort_by(|left, right| left.name.cmp(&right.name));
        deck_names.sort();

        NormalizedState {
            screen_type,
            room_type,
            room_phase,
            character,
            seed,
            ascension_level,
            floor,
            current_hp,
            max_hp,
            gold,
            energy,
            block,
            powers,
            hand,
            monsters,
            card_reward_choices,
            boss_relic_choices,
            event_id: event.id,
            event_name: event.name,
            event_body: event.body,
            event_choices: event.choices,
            relics,
            potions,
            deck_names,
            incoming_damage,
            rest_options,
            danger,
            skip_available,
            shop_cards: shop.cards,
            shop_relics: shop.relics,
            shop_potions: shop.potions,
            purge_available: shop.purge_available,
            purge_cost: shop.purge_cost,
            turn_number,
            orbs,
            stance,
            hand_cards,
            draw_pile,
            discard_pile,
            exhaust_cards,
            master_cards,
            map_nodes,
            map_first_node_chosen,
            map_current_x,
            map_current_y,
            hand_select_max_cards: hand_select.max_cards,
            hand_select_can_pick_zero: hand_select.can_pick_zero,
            hand_select_selected: hand_select.selected,
            current_action,
            card_in_play,
            grid_cards: grid.cards,
            grid_selected_cards: grid.selected_cards,
            grid_for_upgrade: grid.for_upgrade,
            grid_for_transform: grid.for_transform,
            grid_for_purge: grid.for_purge,
            grid_num_cards: grid.num_cards,
            empty_potion_slots,
        }
    }
}
