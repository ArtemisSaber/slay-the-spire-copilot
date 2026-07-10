use super::*;

#[test]
fn build_hand_select_shows_purpose_with_card_in_play() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::HandSelect),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: Some("ExhaustAction".into()),
        card_in_play: Some(card("Burning Pact", "燃烧契约", 1, "SKILL")),
        hand_select_max_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: hand_select]"));
    assert!(prompt.contains("Exhaust a card"));
    assert!(prompt.contains("燃烧契约"));
}

#[test]
fn build_hand_select_shows_selected_cards() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::HandSelect),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        hand_select_selected: vec![card("Defend_R", "防御", 1, "SKILL")],
        hand_select_max_cards: Some(2),
        hand_select_can_pick_zero: true,
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("已选择"));
    assert!(prompt.contains("防御"));
    assert!(prompt.contains("Can skip: yes"));
}

#[test]
fn build_grid_select_upgrade_shows_purpose() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Grid),
        grid_cards: vec![
            card("Strike_R", "打击", 1, "ATTACK"),
            card("Defend_R", "防御", 1, "SKILL"),
        ],
        grid_for_upgrade: true,
        grid_num_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: grid_select]"));
    assert!(prompt.contains("升级"));
    assert!(prompt.contains("打击"));
}

#[test]
fn build_grid_select_purge_shows_purpose() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Grid),
        grid_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        grid_for_purge: true,
        grid_num_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: grid_select]"));
    assert!(prompt.contains("移除"));
}

#[test]
fn build_grid_select_transform_shows_purpose() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Grid),
        grid_cards: vec![
            card("Strike_R", "打击", 1, "ATTACK"),
            card("Defend_R", "防御", 1, "SKILL"),
        ],
        grid_for_transform: true,
        grid_num_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: grid_select]"));
    assert!(prompt.contains("变化"));
}

#[test]
fn build_grid_select_default_shows_generic_purpose() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Grid),
        grid_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        grid_for_upgrade: false,
        grid_for_transform: false,
        grid_for_purge: false,
        grid_num_cards: Some(2),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: grid_select]"));
    assert!(prompt.contains("选择一张卡牌"));
}

#[test]
fn build_grid_select_falls_back_to_hand_when_grid_cards_empty() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Grid),
        grid_cards: vec![],
        hand: vec![card("Bash", "痛击", 2, "ATTACK")],
        grid_for_upgrade: true,
        grid_num_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("痛击"));
    assert!(prompt.contains("升级"));
}

#[test]
fn build_grid_select_no_num_cards() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Grid),
        grid_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        grid_for_purge: true,
        grid_num_cards: None,
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: grid_select]"));
    assert!(prompt.contains("移除"));
    assert!(!prompt.contains("Select"));
}

#[test]
fn build_hand_select_without_card_in_play() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::HandSelect),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: Some("DiscardAction".into()),
        card_in_play: None,
        hand_select_max_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: hand_select]"));
    assert!(prompt.contains("Discard a card"));
    assert!(!prompt.contains("Card playing"));
}

#[test]
fn build_hand_select_put_on_deck_action() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::HandSelect),
        hand: vec![card("Defend_R", "防御", 1, "SKILL")],
        current_action: Some("PutOnDeckAction".into()),
        card_in_play: None,
        hand_select_max_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("Put a card on top of your draw pile"));
}

#[test]
fn build_hand_select_unknown_action() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::HandSelect),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: Some("SomeUnknownAction".into()),
        card_in_play: None,
        hand_select_max_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("SomeUnknownAction"));
}

#[test]
fn build_hand_select_cannot_skip() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::HandSelect),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: Some("ExhaustAction".into()),
        card_in_play: Some(card("Burning Pact", "燃烧契约", 1, "SKILL")),
        hand_select_max_cards: Some(1),
        hand_select_can_pick_zero: false,
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("Can skip: no"));
}

#[test]
fn build_hand_select_no_current_action() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::HandSelect),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: None,
        card_in_play: None,
        hand_select_max_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: hand_select]"));
    assert!(!prompt.contains("Purpose:"));
    assert!(prompt.contains("打击"));
}

#[test]
fn build_hand_select_no_max_cards() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::HandSelect),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: Some("DiscardAction".into()),
        card_in_play: None,
        hand_select_max_cards: None,
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: hand_select]"));
    assert!(prompt.contains("Discard a card"));
    assert!(!prompt.contains("Max:"));
}
