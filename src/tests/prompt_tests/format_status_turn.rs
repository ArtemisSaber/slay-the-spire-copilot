use super::*;

#[test]
fn turn_status_line_with_powers_and_potions() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: Some(10),
        energy: Some(3),
        powers: vec![PowerInfo {
            id: "Strength".into(),
            name: "力量".into(),
            amount: 3,
        }],
        potions: vec![PotionInfo {
            slot: 0,
            name: "再生药水".into(),
            description: "获得 5 层 再生 。".into(),
            price: None,
            can_use: true,
            can_discard: false,
            requires_target: false,
        }],
        incoming_damage: 0,
        danger: danger_safe(),
        ..test_state()
    };
    let output = turn_status_line(&state, &locale);
    assert!(output.contains("力量(3)"));
    assert!(output.contains("再生药水"));
    assert!(output.contains("50/75"));
}

#[test]
fn turn_status_line_with_incoming_damage_and_block() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: Some(5),
        energy: Some(3),
        incoming_damage: 12,
        danger: danger_safe(),
        ..test_state()
    };
    let output = turn_status_line(&state, &locale);
    assert!(output.contains("12"));
    assert!(output.contains("需格挡"));
}

#[test]
fn turn_status_line_with_incoming_damage_no_block_warning() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: Some(20),
        energy: Some(3),
        incoming_damage: 12,
        danger: danger_safe(),
        ..test_state()
    };
    let output = turn_status_line(&state, &locale);
    assert!(output.contains("12"));
    assert!(!output.contains("需格挡"));
}

#[test]
fn turn_status_line_with_incoming_damage_no_block_at_all() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: None,
        energy: Some(3),
        incoming_damage: 8,
        danger: danger_safe(),
        ..test_state()
    };
    let output = turn_status_line(&state, &locale);
    assert!(output.contains("8"));
    assert!(!output.contains("需格挡"));
}

#[test]
fn turn_status_line_block_zero_incoming_no_warning() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: Some(0),
        energy: Some(3),
        incoming_damage: 12,
        danger: danger_safe(),
        ..test_state()
    };
    let output = turn_status_line(&state, &locale);
    assert!(output.contains("12"));
    assert!(!output.contains("需格挡"));
}
