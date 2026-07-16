use super::*;

#[test]
fn communication_mod_edge_fixtures_do_not_report_false_lethal() {
    let fixtures = edge_fixture_list();
    assert!(
        !fixtures.is_empty(),
        "edge fixture list should not be empty"
    );

    for item in fixtures {
        let label = item["label"].as_str().expect("edge fixture label");
        let filename = item["filename"].as_str().expect("edge fixture filename");
        let raw = load_fixture(filename);
        let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));
        let sequence = find_kill_sequence(&state);

        assert!(
            sequence.is_none(),
            "{label} must not be reported as lethal; got {sequence:?}"
        );
    }
}

#[test]
fn target_required_setup_keeps_target_in_kill_sequence() {
    let raw = load_fixture("kill-scan-edge-target-required-setup.json");
    let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));
    let sequence = find_kill_sequence(&state).expect("fixture should have a real lethal sequence");
    let setup = sequence.first().expect("setup should be the first play");

    assert_eq!(setup.card, "edge-targeted-setup");
    assert_eq!(setup.target, Some(0));
}
