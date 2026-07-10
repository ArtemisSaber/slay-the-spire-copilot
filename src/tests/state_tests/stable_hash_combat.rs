use super::*;

#[test]
fn observation_hash_changes_when_draw_pile_changes() {
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["draw_pile"][0]["uuid"] =
        Value::String("changed-draw".into());

    let state1 = NormalizedState::from_raw(&raw1, test_locale());
    let state2 = NormalizedState::from_raw(&raw2, test_locale());

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_discard_pile_changes() {
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["discard_pile"] = serde_json::json!([
        {"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK","uuid":"discarded-card","upgrades":0}
    ]);

    let state1 = NormalizedState::from_raw(&raw1, test_locale());
    let state2 = NormalizedState::from_raw(&raw2, test_locale());

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_monster_block_changes() {
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["monsters"][0]["block"] = Value::Number(7.into());

    let state1 = NormalizedState::from_raw(&raw1, test_locale());
    let state2 = NormalizedState::from_raw(&raw2, test_locale());

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_monster_power_changes() {
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["monsters"][0]["powers"][0]["amount"] =
        Value::Number(9.into());

    let state1 = NormalizedState::from_raw(&raw1, test_locale());
    let state2 = NormalizedState::from_raw(&raw2, test_locale());

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_card_uuid_changes() {
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["hand"][0]["uuid"] =
        Value::String("changed-hand-card".into());

    let state1 = NormalizedState::from_raw(&raw1, test_locale());
    let state2 = NormalizedState::from_raw(&raw2, test_locale());

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}
