use super::*;

fn comparison_fixture() -> (
    AutoPlaySession,
    NormalizedState,
    Vec<ActionCandidate>,
    CardCandidateSelection,
) {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "floor": 12,
            "character": "IRONCLAD",
            "relics": [{
                "id": "Bag of Marbles",
                "name": "Bag of Marbles",
                "description": "At the start of each combat, apply 1 Vulnerable."
            }],
            "deck": [
                {
                    "id": "Strike_R",
                    "name": "Strike",
                    "cost": 1,
                    "type": "ATTACK",
                    "description": "Deal 6 damage."
                },
                {
                    "id": "Bash",
                    "name": "Bash",
                    "cost": 2,
                    "type": "ATTACK",
                    "description": "Deal 8 damage. Apply 2 Vulnerable."
                }
            ],
            "screen_state": {
                "skip_available": true,
                "cards": [
                    {
                        "id": "Uppercut",
                        "name": "Uppercut",
                        "cost": 2,
                        "type": "ATTACK",
                        "description": "Deal 13 damage. Apply 1 Weak and 1 Vulnerable."
                    },
                    {
                        "id": "Shrug It Off",
                        "name": "Shrug It Off",
                        "cost": 1,
                        "type": "SKILL",
                        "description": "Gain 8 Block. Draw 1 card."
                    }
                ]
            }
        }
    });
    let session = AutoPlaySession::default();
    let command_state = command_state(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &session,
        &command_state,
        &state,
    );
    let selected = CardCandidateSelection {
        candidate_index: 1,
        choice_index: 1,
        memory_ids_used: vec![],
    };
    (session, state, candidates, selected)
}

#[test]
fn comparison_uses_opaque_refs_and_reverses_order_with_entropy() {
    let (session, state, candidates, selected) = comparison_fixture();
    let mut entropy_a = [0_u8; 32];
    entropy_a[0] = 2;
    let mut entropy_b = entropy_a;
    entropy_b[0] = 3;

    let case_a = build_card_comparison_case_with_entropy(
        &session,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        &selected,
        None,
        entropy_a,
    )
    .unwrap();
    let case_b = build_card_comparison_case_with_entropy(
        &session,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        &selected,
        None,
        entropy_b,
    )
    .unwrap();
    let payload_a: Value = serde_json::from_str(&case_a.prompt).unwrap();
    let payload_b: Value = serde_json::from_str(&case_b.prompt).unwrap();
    let states_a = payload_a["resulting_states"].as_array().unwrap();
    let states_b = payload_b["resulting_states"].as_array().unwrap();

    for resulting_state in states_a {
        let state_ref = resulting_state["ref"].as_str().unwrap();
        assert!(state_ref.starts_with("resulting_state_"));
        assert!(!state_ref.contains("take"));
        assert!(!state_ref.contains("skip"));
        assert!(!state_ref.contains("added"));
        assert!(!state_ref.contains("unchanged"));
        assert!(resulting_state.get("outcome").is_none());
    }
    assert_ne!(states_a[0]["ref"], states_a[1]["ref"]);
    assert_ne!(
        states_a[0]["deck"].as_array().unwrap().len(),
        states_a[1]["deck"].as_array().unwrap().len()
    );
    assert_eq!(
        states_a[0]["deck"].as_array().unwrap().len(),
        states_b[1]["deck"].as_array().unwrap().len()
    );
    assert_eq!(
        states_a[1]["deck"].as_array().unwrap().len(),
        states_b[0]["deck"].as_array().unwrap().len()
    );
    assert!(!case_a.prompt.contains("Deck A"));
    assert!(!case_a.prompt.contains("Deck B"));
}

#[test]
fn comparison_response_maps_refs_without_exposing_outcome_names() {
    let (session, state, candidates, selected) = comparison_fixture();
    let case = build_card_comparison_case_with_entropy(
        &session,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        &selected,
        None,
        [7_u8; 32],
    )
    .unwrap();

    let preferred = parse_card_comparison_response(
        &format!(
            r#"{{
                "schema_version": 1,
                "verdict": "prefer",
                "preferred_ref": "{}",
                "reason": "Higher projected win chance.",
                "risk": "",
                "memory_ids_used": []
            }}"#,
            case.added_card_ref
        ),
        &case,
        &[],
    )
    .unwrap();
    assert_eq!(preferred.verdict, CardComparisonVerdict::PreferAddedCard);

    for (verdict, expected) in [
        ("indifferent", CardComparisonVerdict::Indifferent),
        ("uncertain", CardComparisonVerdict::Uncertain),
    ] {
        let parsed = parse_card_comparison_response(
            &format!(
                r#"{{
                    "schema_version": 1,
                    "verdict": "{verdict}",
                    "preferred_ref": null,
                    "reason": "",
                    "risk": "",
                    "memory_ids_used": []
                }}"#
            ),
            &case,
            &[],
        )
        .unwrap();
        assert_eq!(parsed.verdict, expected);
    }
}
