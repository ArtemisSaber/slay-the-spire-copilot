use super::*;

#[test]
fn run_fixtures_deterministic_results() {
    let fixtures: Vec<serde_json::Value> = run_fixture_list();
    assert!(
        !fixtures.is_empty(),
        "run fixtures list should not be empty"
    );

    let mut passes = 0usize;
    let mut failures = 0usize;

    for item in &fixtures {
        let label = item["label"].as_str().unwrap_or("?");
        let expected_kill = item["expected_kill"].as_bool().unwrap_or(false);
        let filename = item["filename"].as_str().unwrap_or("");

        let raw = load_fixture(filename);
        let state: NormalizedState =
            serde_json::from_value(raw.clone()).expect("failed to deserialize fixture");
        let actual_kill = can_end_fight(&state);

        if actual_kill == expected_kill {
            passes += 1;
        } else {
            failures += 1;
            eprintln!("FAIL {label}: expected_kill={expected_kill} actual_kill={actual_kill}");
            let hand = &state.hand;
            eprintln!(
                "  Hand: {} cards, energy={}",
                hand.len(),
                state.energy.unwrap_or(-1)
            );
            for c in hand {
                eprintln!(
                    "    {} cost={} type={} playable={}",
                    c.name, c.cost, c.card_type, c.playable
                );
            }
            for m in &state.monsters {
                eprintln!(
                    "    Monster: {} hp={:?} block={:?}",
                    m.name, m.current_hp, m.block
                );
            }
        }
    }

    assert_eq!(
        failures, 0,
        "expected 0 mismatches, got {failures} (passed {passes})"
    );
    assert!(passes > 0, "should have tested at least one fixture");
}
