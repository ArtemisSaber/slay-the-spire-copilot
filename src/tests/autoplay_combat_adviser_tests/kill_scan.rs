use super::*;

#[test]
fn kill_scan_finds_lethal_and_returns_first_play() {
    let state = load_normalized_state("kill-scan-run-pos-hp1-4hand.json");
    let action = try_kill_scan_action(&state).expect("should find a lethal action in pos fixture");
    assert!(matches!(action, AutoPlayAction::Play { .. }));
}

#[test]
fn kill_scan_no_lethal_returns_none() {
    let state = load_normalized_state("kill-scan-run-neg-hp19-energy0.json");
    assert_eq!(try_kill_scan_action(&state), None);
}

#[test]
fn kill_scan_non_combat_screen_returns_none() {
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        ..NormalizedState::default()
    };
    assert_eq!(try_kill_scan_action(&state), None);
}
