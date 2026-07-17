use crate::finalize_run_once;
use crate::gate::{CombatTurnGate, MapGate, SCREEN_CONFIG, should_generate_advice};
use crate::runtime::{
    has_monsters, is_game_over_state, is_non_retryable_command_error, run_end_reason,
    runtime_options_from, should_end_run,
};
use serde_json::json;

fn make_state(screen_type: &str, monsters: Option<Vec<serde_json::Value>>) -> serde_json::Value {
    let mut state = json!({
        "in_game": true,
        "game_state": {
            "screen_type": screen_type,
            "action_phase": "WAITING_ON_USER",
            "floor": 1,
            "room_type": "MonsterRoom",
        }
    });
    if let Some(monster_list) = monsters {
        state["game_state"]["combat_state"] = json!({
            "monsters": monster_list,
            "turn": 1,
        });
    }
    state
}

fn active_monster() -> serde_json::Value {
    json!({"id": "JawWorm", "name": "Jaw Worm", "is_gone": false})
}

fn gone_monster() -> serde_json::Value {
    json!({"id": "JawWorm", "name": "Jaw Worm", "is_gone": true})
}

fn no_combat_state(screen_type: &str) -> serde_json::Value {
    make_state(screen_type, None)
}

fn event_state_with_choices(choices: Vec<&str>) -> serde_json::Value {
    json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "floor": 8,
            "room_type": "EventRoom",
            "screen_state": {
                "text": "A strange event appears.",
                "options": choices
            }
        }
    })
}

fn with_monsters(screen_type: &str) -> serde_json::Value {
    make_state(screen_type, Some(vec![active_monster()]))
}

fn without_monsters(screen_type: &str) -> serde_json::Value {
    make_state(screen_type, Some(vec![]))
}

fn menu_state() -> serde_json::Value {
    json!({"in_game": false})
}

fn map_json(first_node_chosen: bool, children: Vec<(i64, i64)>) -> serde_json::Value {
    let children_json: Vec<serde_json::Value> = children
        .iter()
        .map(|(x, y)| json!({"x": x, "y": y}))
        .collect();
    json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "room_phase": "COMPLETE",
            "floor": 5,
            "room_type": "MonsterRoom",
            "screen_state": {
                "first_node_chosen": first_node_chosen,
                "current_node": {"x": 1, "y": 2}
            },
            "map": [
                {"symbol": "M", "x": 1, "y": 2, "children": children_json},
                {"symbol": "E", "x": 3, "y": 3, "children": []},
                {"symbol": "?", "x": 4, "y": 3, "children": []},
                {"symbol": "M", "x": 5, "y": 3, "children": []}
            ]
        }
    })
}

fn map_nodes_for(children: Vec<(i64, i64)>) -> Vec<crate::state::MapCoord> {
    vec![
        crate::state::MapCoord {
            symbol: "M".into(),
            x: 1,
            y: 2,
            children,
        },
        crate::state::MapCoord {
            symbol: "E".into(),
            x: 3,
            y: 3,
            children: vec![],
        },
        crate::state::MapCoord {
            symbol: "?".into(),
            x: 4,
            y: 3,
            children: vec![],
        },
        crate::state::MapCoord {
            symbol: "M".into(),
            x: 5,
            y: 3,
            children: vec![],
        },
    ]
}

fn map_nodes() -> Vec<crate::state::MapCoord> {
    map_nodes_for(vec![(3, 3), (4, 3), (5, 3)])
}

#[path = "main_tests/finalization.rs"]
mod finalization;
#[path = "main_tests/gating.rs"]
mod gating;
#[path = "main_tests/input.rs"]
mod input;
#[path = "main_tests/map.rs"]
mod map;
#[path = "main_tests/runtime.rs"]
mod runtime;
