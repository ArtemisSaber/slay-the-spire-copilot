use super::*;

#[test]
fn all_fixtures_parse() {
    let names = fixture_list();
    assert!(!names.is_empty());
    for name in &names {
        let raw = load_fixture(name);
        let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));
        assert_eq!(
            state.screen_type.as_ref().map(|st| st.as_str()),
            Some("NONE"),
            "{name}"
        );
        assert!(!state.hand.is_empty(), "{name}");
        assert!(!state.monsters.is_empty(), "{name}");
    }
}

#[test]
fn l52_two_card_kill() {
    let raw = load_fixture(&fixture_by_line("L52"));
    let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));

    // Strike(13) + Twin Strike+(14*2=28) = 41 > 26hp, energy=2
    assert!(can_end_fight(&state));
    let seq = find_kill_sequence(&state).unwrap();
    assert!(!seq.is_empty());
}

#[test]
fn l156_xcost_aoe_kill() {
    let raw = load_fixture(&fixture_by_line("L156"));
    let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));

    // Whirlwind+ X-cost (25 per X), energy=5, 36hp Giant Head
    assert!(can_end_fight(&state));
    let seq = find_kill_sequence(&state).unwrap();
    assert!(!seq.is_empty());
}

#[test]
fn l157_single_strike_kill() {
    let raw = load_fixture(&fixture_by_line("L157"));
    let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));

    // Any Strike (40 dmg) kills 33hp Giant Head; DFS may find multi-card path first
    assert!(can_end_fight(&state));
    let seq = find_kill_sequence(&state).unwrap();
    assert!(!seq.is_empty(), "should find some kill sequence");
}

#[test]
fn l208_minion_end() {
    let raw = load_fixture(&fixture_by_line("L208"));
    let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));

    // Strike kills 6hp snake leader → minion dagger remains but combat ends
    assert!(can_end_fight(&state));
    let seq = find_kill_sequence(&state).unwrap();
    assert!(!seq.is_empty());
}

#[test]
fn non_combat_screen_returns_none() {
    let raw = load_fixture(&fixture_by_line("L52"));
    let mut state = NormalizedState::from_raw(&raw, &Locale::load("zh"));
    state.screen_type = Some(crate::state::ScreenType::CardReward);

    assert!(!can_end_fight(&state));
    assert!(find_kill_sequence(&state).is_none());
}

#[test]
fn empty_hand_returns_none() {
    let raw = load_fixture(&fixture_by_line("L52"));
    let mut state = NormalizedState::from_raw(&raw, &Locale::load("zh"));
    state.hand.clear();

    assert!(!can_end_fight(&state));
    assert!(find_kill_sequence(&state).is_none());
}

#[test]
fn pure_block_hand_returns_none() {
    let raw = load_fixture(&fixture_by_line("L52"));
    let mut state = NormalizedState::from_raw(&raw, &Locale::load("zh"));

    state.hand = vec![crate::state::CardInfo {
        id: "Defend_R".into(),
        name: "防御".into(),
        cost: 1,
        card_type: "SKILL".into(),
        upgraded: false,
        uuid: Some("defend-1".into()),
        description: "获得 5 点 格挡 。".into(),
        price: None,
        playable: true,
        has_target: false,
    }];

    assert!(!can_end_fight(&state));
    assert!(find_kill_sequence(&state).is_none());
}
