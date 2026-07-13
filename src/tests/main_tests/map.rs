use super::*;

#[test]
fn map_gate_act_entry_generates() {
    let mut gate = MapGate::new();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "room_phase": "COMPLETE",
            "screen_state": {
                "first_node_chosen": false,
                "current_node": {"x": -1, "y": 15}
            }
        }
    });
    assert!(gate.should_generate(&raw, &map_nodes()));
}

#[test]
fn map_gate_crossroads_generates() {
    let mut gate = MapGate::new();
    let raw = map_json(true, vec![(3, 3), (4, 3)]);
    let nodes = map_nodes_for(vec![(3, 3), (4, 3)]);
    assert!(gate.should_generate(&raw, &nodes));
}

#[test]
fn map_gate_single_child_skips() {
    let mut gate = MapGate::new();
    let raw = map_json(true, vec![(3, 3)]);
    let nodes = map_nodes_for(vec![(3, 3)]);
    assert!(!gate.should_generate(&raw, &nodes));
}

#[test]
fn map_gate_same_crossroads_skips() {
    let mut gate = MapGate::new();
    let raw = map_json(true, vec![(3, 3), (4, 3)]);
    let nodes = map_nodes_for(vec![(3, 3), (4, 3)]);
    assert!(gate.should_generate(&raw, &nodes));
    gate.record(&nodes, 1, 2);
    assert!(!gate.should_generate(&raw, &nodes));
}

#[test]
fn map_gate_different_crossroads_generates() {
    let mut gate = MapGate::new();
    let raw1 = map_json(true, vec![(3, 3)]);
    let nodes1 = map_nodes_for(vec![(3, 3)]);
    assert!(!gate.should_generate(&raw1, &nodes1));
    let raw2 = map_json(true, vec![(3, 3), (4, 3)]);
    let nodes2 = map_nodes_for(vec![(3, 3), (4, 3)]);
    assert!(gate.should_generate(&raw2, &nodes2));
}

#[test]
fn map_gate_room_phase_not_complete_skips() {
    let mut gate = MapGate::new();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "room_phase": "NORMAL",
            "screen_state": {
                "first_node_chosen": true,
                "current_node": {"x": 1, "y": 2}
            }
        }
    });
    assert!(!gate.should_generate(&raw, &map_nodes()));
}

#[test]
fn map_gate_no_current_node_skips() {
    let mut gate = MapGate::new();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "room_phase": "COMPLETE",
            "screen_state": {
                "first_node_chosen": true
            }
        }
    });
    assert!(!gate.should_generate(&raw, &map_nodes()));
}

#[test]
fn map_gate_resets_shop_visited_on_act_entry() {
    let mut gate = MapGate::new();
    gate.on_shop();
    assert!(gate.shop_visited);
    gate.on_act_entry();
    assert!(!gate.shop_visited);
}

#[test]
fn map_gate_sets_shop_visited_on_shop_screen() {
    let mut gate = MapGate::new();
    assert!(!gate.shop_visited);
    gate.on_shop();
    assert!(gate.shop_visited);
}

#[test]
fn map_not_in_simple_config_gating() {
    let state = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "room_phase": "COMPLETE",
            "floor": 5
        }
    });
    assert!(!should_generate_advice("MAP", &state));
}
