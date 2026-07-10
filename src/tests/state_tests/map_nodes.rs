use super::*;

#[test]
fn map_nodes_parsed_from_state() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {},
            "map": [
                {
                    "symbol": "M",
                    "x": 0,
                    "y": 0,
                    "children": [{"x": 1, "y": 1}]
                },
                {
                    "symbol": "R",
                    "x": 1,
                    "y": 1,
                    "children": []
                }
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.map_nodes.len(), 2);
    assert_eq!(state.map_nodes[0].symbol, "M");
    assert_eq!(state.map_nodes[0].x, 0);
    assert_eq!(state.map_nodes[0].y, 0);
    assert_eq!(state.map_nodes[0].children, vec![(1, 1)]);
    assert_eq!(state.map_nodes[1].symbol, "R");
    assert_eq!(state.map_nodes[1].x, 1);
    assert_eq!(state.map_nodes[1].y, 1);
    assert!(state.map_nodes[1].children.is_empty());
}

#[test]
fn map_screen_state_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {
                "first_node_chosen": true,
                "current_node": {"x": 3, "y": 7}
            },
            "map": [
                {"symbol": "M", "x": 3, "y": 7, "children": []}
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.map_first_node_chosen, Some(true));
    assert_eq!(state.map_current_x, Some(3));
    assert_eq!(state.map_current_y, Some(7));
}

#[test]
fn map_screen_state_defaults_to_none() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.map_first_node_chosen, None);
    assert_eq!(state.map_current_x, None);
    assert_eq!(state.map_current_y, None);
}

#[test]
fn map_nodes_missing_children_defaults_to_empty() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {},
            "map": [
                {"symbol": "M", "x": 3, "y": 7}
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.map_nodes.len(), 1);
    assert_eq!(state.map_nodes[0].symbol, "M");
    assert_eq!(state.map_nodes[0].x, 3);
    assert_eq!(state.map_nodes[0].y, 7);
    assert!(state.map_nodes[0].children.is_empty());
}

#[test]
fn map_nodes_missing_coords_default_to_zero() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {},
            "map": [
                {"symbol": "R"}
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.map_nodes.len(), 1);
    assert_eq!(state.map_nodes[0].x, 0);
    assert_eq!(state.map_nodes[0].y, 0);
    assert_eq!(state.map_nodes[0].symbol, "R");
}

#[test]
fn map_nodes_missing_symbol_defaults_to_question() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {},
            "map": [
                {"x": 1, "y": 2}
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.map_nodes[0].symbol, "?");
}

#[test]
fn map_nodes_child_missing_coords_filtered_out() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {},
            "map": [
                {
                    "symbol": "M",
                    "x": 0,
                    "y": 0,
                    "children": [
                        {"x": 1, "y": 1},
                        {"y": 2},
                        {"x": 3}
                    ]
                }
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.map_nodes.len(), 1);
    assert_eq!(state.map_nodes[0].children, vec![(1, 1)]);
}
