use super::*;

#[test]
fn grid_for_upgrade_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "GRID",
            "screen_state": {
                "cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"},
                    {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL"}
                ],
                "for_upgrade": true,
                "for_transform": false,
                "for_purge": false,
                "num_cards": 2
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(state.grid_for_upgrade);
    assert!(!state.grid_for_transform);
    assert!(!state.grid_for_purge);
    assert_eq!(state.grid_cards.len(), 2);
    assert_eq!(state.grid_num_cards, Some(2));
}

#[test]
fn grid_for_transform_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "GRID",
            "screen_state": {
                "cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "for_upgrade": false,
                "for_transform": true,
                "for_purge": false,
                "num_cards": 1
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(!state.grid_for_upgrade);
    assert!(state.grid_for_transform);
    assert!(!state.grid_for_purge);
    assert_eq!(state.grid_cards.len(), 1);
    assert_eq!(state.grid_num_cards, Some(1));
}

#[test]
fn grid_for_purge_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "GRID",
            "screen_state": {
                "cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"},
                    {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL"},
                    {"id": "Bash", "name": "Bash", "cost": 2, "type": "ATTACK"}
                ],
                "for_upgrade": false,
                "for_transform": false,
                "for_purge": true,
                "num_cards": 3,
                "selected_cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ]
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(!state.grid_for_upgrade);
    assert!(!state.grid_for_transform);
    assert!(state.grid_for_purge);
    assert_eq!(state.grid_cards.len(), 3);
    assert_eq!(state.grid_num_cards, Some(3));
    assert_eq!(state.grid_selected_cards.len(), 1);
    assert_eq!(state.grid_selected_cards[0].id, "Strike_R");
}

#[test]
fn grid_all_flags_false_by_default() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "GRID",
            "screen_state": {
                "cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ]
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(!state.grid_for_upgrade);
    assert!(!state.grid_for_transform);
    assert!(!state.grid_for_purge);
    assert!(state.grid_num_cards.is_none());
    assert!(state.grid_selected_cards.is_empty());
}

#[test]
fn hand_select_can_pick_zero_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {
                "max_cards": 1,
                "can_pick_zero": true,
                "selected": []
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.hand_select_max_cards, Some(1));
    assert!(state.hand_select_can_pick_zero);
    assert!(state.hand_select_selected.is_empty());
}

#[test]
fn hand_select_can_pick_zero_defaults_to_false() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {
                "max_cards": 2,
                "selected": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ]
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.hand_select_max_cards, Some(2));
    assert!(!state.hand_select_can_pick_zero);
    assert_eq!(state.hand_select_selected.len(), 1);
    assert_eq!(state.hand_select_selected[0].id, "Strike_R");
}
