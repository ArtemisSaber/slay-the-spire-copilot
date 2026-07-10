use super::*;

#[test]
fn build_map_crossroad_shows_next_nodes() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3), (2, 3)]),
            make_node("E", 1, 3, vec![(1, 4)]),
            make_node("?", 2, 3, vec![(2, 4)]),
            make_node("R", 1, 4, vec![]),
            make_node("R", 2, 4, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("(左侧)"));
    assert!(prompt.contains("(右侧)"));
    assert!(!prompt.contains("(中间)"));
}

#[test]
fn build_map_crossroad_no_full_coordinate_routes() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("E", 1, 3, vec![(1, 4)]),
            make_node("R", 1, 4, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(!prompt.contains("(1,"));
    assert!(!prompt.contains("(1,2)"));
    assert!(!prompt.contains("(1,3)"));
}

#[test]
fn build_map_crossroad_includes_ahead_chain() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("E", 1, 3, vec![(1, 4)]),
            make_node("R", 1, 4, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("Ahead"));
}

#[test]
fn build_map_crossroad_includes_annotations() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("E", 1, 3, vec![(1, 4)]),
            make_node("M", 1, 4, vec![(1, 5)]),
            make_node("M", 1, 5, vec![(1, 6)]),
            make_node("M", 1, 6, vec![(1, 7)]),
            make_node("R", 1, 7, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("E→R gap"));
}

#[test]
fn build_map_crossroad_softens_no_shop_when_shop_visited() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("M", 1, 3, vec![(1, 4)]),
            make_node("R", 1, 4, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, true);
    assert!(prompt.contains("No shop ahead"));
}

#[test]
fn build_map_crossroad_keeps_no_shop_when_not_visited() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("M", 1, 3, vec![(1, 4)]),
            make_node("R", 1, 4, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("✗ No shop"));
    assert!(!prompt.contains("No shop ahead"));
}

#[test]
fn build_map_crossroad_single_child_direct() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("R", 1, 3, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("(唯一)"));
}

#[test]
fn build_map_crossroad_unknown_symbol() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("X", 1, 3, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("X(X)"));
}

#[test]
fn build_map_crossroad_current_position_not_found() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![make_node("M", 0, 0, vec![])],
        map_first_node_chosen: Some(true),
        map_current_x: Some(99),
        map_current_y: Some(99),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("[mode: map_crossroad]"));
    assert!(!prompt.contains("Ahead"));
}

#[test]
fn build_map_crossroad_many_candidates_limited() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node(
                "M",
                0,
                0,
                vec![(0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1)],
            ),
            make_node("M", 0, 1, vec![]),
            make_node("?", 1, 1, vec![]),
            make_node("R", 2, 1, vec![]),
            make_node("$", 3, 1, vec![]),
            make_node("E", 4, 1, vec![]),
            make_node("T", 5, 1, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(0),
        map_current_y: Some(0),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert_eq!(prompt.matches("Candidate ").count(), 5);
    assert!(prompt.contains("Candidate 5"));
    assert!(!prompt.contains("Candidate 6"));
}
