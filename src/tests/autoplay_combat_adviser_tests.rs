use super::*;
use crate::state::ScreenType;
use crate::test_utils;

fn load_normalized_state(filename: &str) -> NormalizedState {
    let raw = test_utils::load_fixture(filename);
    serde_json::from_value(raw).expect("fixture should deserialize as NormalizedState")
}

fn combat_state_with_cards(energy: i64, cards: Vec<serde_json::Value>) -> NormalizedState {
    let monsters = serde_json::json!([{
        "name": "Jaw Worm",
        "index": 0,
        "current_hp": 20,
        "max_hp": 20,
        "block": 0,
        "intent": "ATTACK",
        "damage": 5,
        "hits": 1,
        "monster_powers": [],
        "can_be_killed": false,
        "is_scaling": false
    }]);
    let payload = serde_json::json!({
        "screen_type": "NONE",
        "energy": energy,
        "hand": cards,
        "monsters": monsters,
        "incoming_damage": 5,
        "powers": [],
        "relics": [],
        "potions": [],
        "draw_pile": [],
        "discard_pile": [],
        "deck_names": []
    });
    serde_json::from_value(payload).expect("should deserialize as NormalizedState")
}

fn strike_card(uuid: &str, cost: i64) -> serde_json::Value {
    serde_json::json!({
        "id": "Strike_R",
        "name": "Strike",
        "cost": cost,
        "card_type": "ATTACK",
        "uuid": uuid,
        "description": "Deal 6 damage.",
        "has_target": true,
        "playable": true
    })
}

fn defend_card() -> serde_json::Value {
    serde_json::json!({
        "id": "Defend_R",
        "name": "Defend",
        "cost": 1,
        "card_type": "SKILL",
        "uuid": "def-1",
        "description": "Gain 5 Block.",
        "has_target": false,
        "playable": true
    })
}

fn limit_break_card() -> serde_json::Value {
    serde_json::json!({
        "id": "Limit Break",
        "name": "Limit Break",
        "cost": 1,
        "card_type": "SKILL",
        "uuid": "lb-1",
        "description": "Double your Strength.",
        "has_target": false,
        "playable": true
    })
}

fn thunderclap_card() -> serde_json::Value {
    serde_json::json!({
        "id": "Thunderclap",
        "name": "Thunderclap",
        "cost": 1,
        "card_type": "ATTACK",
        "uuid": "tc-1",
        "description": "Deal 4 damage and apply 1 Vulnerable to ALL enemies.",
        "has_target": false,
        "playable": true
    })
}

fn two_monster_state(energy: i64, cards: Vec<serde_json::Value>) -> NormalizedState {
    let monsters = serde_json::json!([
        {
            "name": "Slime A",
            "index": 0,
            "current_hp": 15,
            "max_hp": 20,
            "block": 0,
            "intent": "ATTACK",
            "damage": 5,
            "hits": 1,
            "monster_powers": [],
            "can_be_killed": false,
            "is_scaling": false
        },
        {
            "name": "Slime B",
            "index": 1,
            "current_hp": 15,
            "max_hp": 20,
            "block": 0,
            "intent": "ATTACK",
            "damage": 5,
            "hits": 1,
            "monster_powers": [],
            "can_be_killed": false,
            "is_scaling": false
        }
    ]);
    let payload = serde_json::json!({
        "screen_type": "NONE",
        "energy": energy,
        "hand": cards,
        "monsters": monsters,
        "incoming_damage": 5,
        "powers": [],
        "relics": [],
        "potions": [],
        "draw_pile": [],
        "discard_pile": [],
        "deck_names": []
    });
    serde_json::from_value(payload).expect("should deserialize as NormalizedState")
}

#[path = "autoplay_combat_adviser_tests/kill_scan.rs"]
mod kill_scan;
#[path = "autoplay_combat_adviser_tests/ranking.rs"]
mod ranking;
#[path = "autoplay_combat_adviser_tests/targeting.rs"]
mod targeting;
