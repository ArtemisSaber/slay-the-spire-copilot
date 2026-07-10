use super::*;

#[test]
fn stable_hash_same_state_same_hash() {
    let raw = load_fixture("combat-state.json");
    let state1 = NormalizedState::from_raw(&raw, test_locale());
    let state2 = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state1.stable_hash(), state2.stable_hash());
}

#[test]
fn stable_hash_different_state_different_hash() {
    let combat = load_fixture("combat-state.json");
    let reward = load_fixture("card-reward-state.json");

    let state1 = NormalizedState::from_raw(&combat, test_locale());
    let state2 = NormalizedState::from_raw(&reward, test_locale());

    assert_ne!(state1.stable_hash(), state2.stable_hash());
}

#[test]
fn stable_hash_produces_hex() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    let hash = state.stable_hash();

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}
