use super::*;

fn reward_fixture() -> (
    AutoPlaySession,
    CommandState,
    NormalizedState,
    Vec<ActionCandidate>,
) {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "floor": 7,
            "gold": 120,
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
    (session, command_state, state, candidates)
}

#[test]
fn candidate_selector_prompt_is_forced_pick_and_omits_skip() {
    let (session, _, state, candidates) = reward_fixture();

    let prompt = build_card_candidate_prompt(
        &session,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        None,
    )
    .unwrap();
    let payload: Value = serde_json::from_str(&prompt).unwrap();

    assert!(
        payload["task"]
            .as_str()
            .unwrap()
            .contains("Assume exactly one offered card must be added")
    );
    assert_eq!(payload["offered_cards"].as_array().unwrap().len(), 2);
    assert_eq!(payload["offered_cards"][0]["ref"], "A0");
    assert_eq!(payload["offered_cards"][1]["ref"], "A1");
    assert_eq!(
        payload["offered_cards"][1]["card"]["description"],
        "Gain 8 Block. Draw 1 card."
    );
    assert!(
        payload
            .pointer("/scenario/card_reward/skip_available")
            .is_none()
    );
    assert!(
        payload
            .pointer("/scenario/card_reward/selection_policy")
            .is_none()
    );
    assert!(!prompt.contains("\"kind\": \"skip\""));
    assert!(!prompt.contains("marginal_net_gain"));
}

#[test]
fn candidate_selector_response_resolves_only_offered_cards() {
    let (_, _, _, candidates) = reward_fixture();

    let selected = parse_card_candidate_response(
        r#"{
            "schema_version": 1,
            "selected_ref": "A1",
            "reason": "Adds block and draw.",
            "risk": "",
            "memory_ids_used": []
        }"#,
        &candidates,
        &[],
    )
    .unwrap();

    assert_eq!(selected.candidate_index, 1);
    assert_eq!(selected.choice_index, 1);

    let error = parse_card_candidate_response(
        r#"{
            "schema_version": 1,
            "selected_ref": "A2",
            "reason": "",
            "risk": "",
            "memory_ids_used": []
        }"#,
        &candidates,
        &[],
    )
    .unwrap_err();
    assert!(error.to_string().contains("not an offered card"));
}
