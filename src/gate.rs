use crate::runtime::has_monsters;
use crate::state::MapCoord;
use std::collections::HashMap;

pub struct ScreenConfig {
    pub generate: &'static [&'static str],
    pub generate_on_combat: &'static [&'static str],
}

pub const SCREEN_CONFIG: ScreenConfig = ScreenConfig {
    generate: &["CARD_REWARD", "BOSS_REWARD", "EVENT", "REST"],
    generate_on_combat: &[],
};

pub struct CombatTurnGate {
    last_turn: Option<(String, i64)>,
}

impl CombatTurnGate {
    pub fn new() -> Self {
        CombatTurnGate { last_turn: None }
    }

    pub fn is_player_turn_start(&mut self, raw: &serde_json::Value) -> bool {
        if raw
            .pointer("/game_state/screen_type")
            .and_then(|v| v.as_str())
            != Some("NONE")
        {
            return false;
        }
        if raw
            .pointer("/game_state/action_phase")
            .and_then(|v| v.as_str())
            != Some("WAITING_ON_USER")
        {
            return false;
        }
        if !has_monsters(raw) {
            return false;
        }
        let identity = match combat_identity(raw) {
            Some(id) => id,
            None => return false,
        };
        let turn = match raw
            .pointer("/game_state/combat_state/turn")
            .and_then(|v| v.as_i64())
        {
            Some(t) => t,
            None => return false,
        };
        let key = (identity, turn);
        if self.last_turn.as_ref() == Some(&key) {
            return false;
        }
        self.last_turn = Some(key);
        true
    }
}

pub struct MapGate {
    pub last_advised_next: Option<Vec<(i64, i64)>>,
    pub shop_visited: bool,
}

impl MapGate {
    pub fn new() -> Self {
        MapGate {
            last_advised_next: None,
            shop_visited: false,
        }
    }

    pub fn should_generate(&mut self, raw: &serde_json::Value, map_nodes: &[MapCoord]) -> bool {
        let rp = raw
            .pointer("/game_state/room_phase")
            .and_then(|v| v.as_str());
        if rp != Some("COMPLETE") {
            return false;
        }

        let first_chosen = raw
            .pointer("/game_state/screen_state/first_node_chosen")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if !first_chosen {
            return true;
        }

        let current = raw
            .pointer("/game_state/screen_state/current_node")
            .and_then(|v| Some((v.get("x")?.as_i64()?, v.get("y")?.as_i64()?)));

        let Some((cx, cy)) = current else {
            return false;
        };

        let node_map: HashMap<(i64, i64), &MapCoord> =
            map_nodes.iter().map(|n| ((n.x, n.y), n)).collect();

        let Some(node) = node_map.get(&(cx, cy)) else {
            return false;
        };

        let mut next_coords: Vec<(i64, i64)> = node
            .children
            .iter()
            .filter(|(x, y)| node_map.contains_key(&(*x, *y)))
            .copied()
            .collect();
        next_coords.sort();

        if next_coords.len() <= 1 {
            return false;
        }

        if self.last_advised_next.as_ref() == Some(&next_coords) {
            return false;
        }

        true
    }

    pub fn record(&mut self, map_nodes: &[MapCoord], current_x: i64, current_y: i64) {
        let node_map: HashMap<(i64, i64), &MapCoord> =
            map_nodes.iter().map(|n| ((n.x, n.y), n)).collect();

        if let Some(node) = node_map.get(&(current_x, current_y)) {
            let mut next: Vec<(i64, i64)> = node
                .children
                .iter()
                .filter(|(x, y)| node_map.contains_key(&(*x, *y)))
                .copied()
                .collect();
            next.sort();
            self.last_advised_next = Some(next);
        }
    }

    pub fn on_shop(&mut self) {
        self.shop_visited = true;
    }

    pub fn on_act_entry(&mut self) {
        self.shop_visited = false;
    }
}

pub fn combat_identity(raw: &serde_json::Value) -> Option<String> {
    let floor = raw.pointer("/game_state/floor")?.as_i64()?;
    let room_type = raw
        .pointer("/game_state/room_type")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    Some(format!("{floor}:{room_type}"))
}

pub fn should_generate_advice(screen_type: &str, raw: &serde_json::Value) -> bool {
    if screen_type == "EVENT" {
        return available_event_choice_count(raw) > 1;
    }
    if screen_type == "REST" {
        return available_rest_option_count(raw) > 0;
    }
    if SCREEN_CONFIG.generate.contains(&screen_type) {
        return true;
    }
    if SCREEN_CONFIG.generate_on_combat.contains(&screen_type) && has_monsters(raw) {
        return true;
    }
    false
}

pub fn available_event_choice_count(raw: &serde_json::Value) -> usize {
    let choices = raw
        .pointer("/game_state/screen_state/options")
        .or_else(|| raw.pointer("/game_state/screen_state/choices"))
        .or_else(|| raw.pointer("/game_state/screen_state/buttons"))
        .or_else(|| raw.pointer("/game_state/choice_list"));

    choices
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|choice| {
                    !choice
                        .get("disabled")
                        .and_then(|disabled| disabled.as_bool())
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

pub fn available_rest_option_count(raw: &serde_json::Value) -> usize {
    raw.pointer("/game_state/screen_state/rest_options")
        .and_then(|v| v.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0)
}
