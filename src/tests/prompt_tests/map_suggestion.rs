use super::*;

#[test]
fn build_map_suggestion_includes_route_chains_and_counts() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1)]),
            make_node("?", 0, 1, vec![(0, 2)]),
            make_node("R", 0, 2, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(0),
        map_current_y: Some(0),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.contains("Route 1 (唯一)"));
    assert!(prompt.contains("Candidate 1"));
    assert!(prompt.contains("Recommendation label: Route 1 (唯一) — M→?→R"));
    assert!(prompt.contains("M→?→R"));
    assert!(prompt.contains("Monsters:1"));
    assert!(prompt.starts_with("[mode: map_suggestion]"));
}

#[test]
fn build_map_suggestion_multiple_paths_labeled() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1), (1, 1)]),
            make_node("R", 0, 1, vec![(0, 2)]),
            make_node("E", 1, 1, vec![(1, 2)]),
            make_node("?", 0, 2, vec![]),
            make_node("$", 1, 2, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(0),
        map_current_y: Some(0),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.contains("Route 1 (左侧)"));
    assert!(prompt.contains("Route 2 (右侧)"));
    assert!(!prompt.contains("Route 3"));
}

#[test]
fn build_map_suggestion_limits_current_route_candidates() {
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
    let prompt = build_map_suggestion(&state, &locale);
    assert_eq!(prompt.matches("Candidate ").count(), 5);
    assert!(prompt.contains("Candidate 5"));
    assert!(!prompt.contains("Candidate 6"));
    assert!(prompt.contains("Recommendation label: Route"));
}

#[test]
fn build_map_suggestion_root_selection() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(1),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1)]),
            make_node("?", 0, 1, vec![]),
            make_node("E", 1, 0, vec![(1, 1)]),
            make_node("R", 1, 1, vec![]),
        ],
        map_first_node_chosen: Some(false),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.contains("Root 1 (左侧)"));
    assert!(prompt.contains("Root 2 (右侧)"));
    assert!(prompt.contains("Recommendation label: Root"));
}

#[test]
fn build_map_suggestion_limits_root_candidates() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(1),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1)]),
            make_node("?", 0, 1, vec![]),
            make_node("M", 1, 0, vec![(1, 1)]),
            make_node("R", 1, 1, vec![]),
            make_node("M", 2, 0, vec![(2, 1)]),
            make_node("$", 2, 1, vec![]),
            make_node("M", 3, 0, vec![(3, 1)]),
            make_node("E", 3, 1, vec![]),
            make_node("M", 4, 0, vec![(4, 1)]),
            make_node("T", 4, 1, vec![]),
            make_node("M", 5, 0, vec![(5, 1)]),
            make_node("M", 5, 1, vec![]),
        ],
        map_first_node_chosen: Some(false),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert_eq!(prompt.matches("Candidate ").count(), 5);
    assert!(prompt.contains("Candidate 5"));
    assert!(!prompt.contains("Candidate 6"));
}

#[test]
fn build_map_suggestion_includes_status_line() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        character: Some("IRONCLAD".into()),
        floor: Some(5),
        current_hp: Some(62),
        max_hp: Some(75),
        gold: Some(180),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1)]),
            make_node("R", 0, 1, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(0),
        map_current_y: Some(0),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.contains("铁甲战士"));
    assert!(prompt.contains("62/75"));
    assert!(prompt.contains("180"));
}

#[test]
fn build_map_suggestion_empty_paths_graceful() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(5),
        map_nodes: vec![],
        map_first_node_chosen: Some(true),
        map_current_x: Some(99),
        map_current_y: Some(99),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.starts_with("[mode: map_suggestion]"));
}

#[test]
fn build_map_suggestion_root_with_branches() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Map),
        floor: Some(1),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1), (1, 1)]),
            make_node("R", 0, 1, vec![(0, 2)]),
            make_node("?", 1, 1, vec![(1, 2)]),
            make_node("$", 0, 2, vec![]),
            make_node("T", 1, 2, vec![]),
        ],
        map_first_node_chosen: Some(false),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.contains("Root 1 (唯一)"));
    assert_eq!(prompt.matches("Candidate ").count(), 2);
}
