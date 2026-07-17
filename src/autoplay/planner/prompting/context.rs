use serde_json::{Value, json};

use crate::autoplay::control::AutoPlaySession;
use crate::locales::Locale;
use crate::state::{DangerLevel, NormalizedState};

mod actions;
mod map;
mod projection;
mod screens;

pub(super) use actions::{annotate_map_action_positions, prompt_action_candidates};
use projection::{potion_value, power_values, relic_value};
use screens::add_screen_context;

pub(in crate::autoplay::planner) fn structured_scenario(
    session: &AutoPlaySession,
    state: &NormalizedState,
    locale: &Locale,
    shop_visited: bool,
) -> Value {
    let kind = scenario_kind(state);
    let mut scenario = json!({
        "kind": kind,
        "screen_type": state.screen_type.as_ref().map(|screen| screen.as_str()),
        "room_type": state.room_type.as_ref().map(|room| room.as_str()),
        "run": {
            "character": state.character,
            "ascension_level": state.ascension_level,
            "floor": state.floor,
            "gold": state.gold,
        },
        "player": {
            "current_hp": state.current_hp,
            "max_hp": state.max_hp,
            "block": state.block,
            "energy": state.energy,
            "stance": state.stance,
            "powers": power_values(&state.powers),
            "orbs": state.orbs.iter().map(|orb| json!({
                "id": orb.id,
                "amount": orb.amount,
            })).collect::<Vec<_>>(),
        },
        "inventory": {
            "relics": state.relics.iter()
                .map(|relic| relic_value(relic, locale))
                .collect::<Vec<_>>(),
            "potions": state.potions.iter()
                .map(|potion| potion_value(potion, locale))
                .collect::<Vec<_>>(),
            "empty_potion_slots": state.empty_potion_slots,
        },
        "threat": {
            "incoming_damage": state.incoming_damage,
            "hp_critical": state.danger.hp_critical,
            "incoming_lethal": state.danger.incoming_lethal,
            "no_block_against_hit": state.danger.no_block_against_hit,
            "any_monster_attacking": state.danger.any_monster_attacking,
            "wrath_stance": state.danger.wrath_stance,
            "level": danger_level(&state.danger.level),
        },
    });

    add_screen_context(
        scenario
            .as_object_mut()
            .expect("structured scenario must be an object"),
        kind,
        session,
        state,
        locale,
        shop_visited,
    );
    scenario
}

fn scenario_kind(state: &NormalizedState) -> &'static str {
    match state.screen_type.as_ref().map(|screen| screen.as_str()) {
        Some("CARD_REWARD") if state.is_in_combat() => "combat_card_choice",
        Some("CARD_REWARD") if state.is_boss_card_reward() => "boss_card_reward",
        Some("CARD_REWARD") => "card_reward",
        Some("BOSS_REWARD") => "boss_relic",
        Some("COMBAT_REWARD") => "combat_reward",
        Some("REST") => "rest",
        Some("EVENT") => "event",
        Some("SHOP_ROOM" | "SHOP_SCREEN") => "shop",
        Some("MAP") if state.map_first_node_chosen == Some(true) => "map_crossroad",
        Some("MAP") => "map_suggestion",
        Some("HAND_SELECT") => "hand_select",
        Some("GRID") => "grid",
        Some("CHEST") => "chest",
        Some("COMPLETE") => "complete",
        Some("NONE") => "combat",
        _ if !state.monsters.is_empty() => "combat",
        _ => "generic",
    }
}

fn danger_level(level: &DangerLevel) -> &'static str {
    match level {
        DangerLevel::Safe => "safe",
        DangerLevel::Caution => "caution",
        DangerLevel::Danger => "danger",
    }
}
