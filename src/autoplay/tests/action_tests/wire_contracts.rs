use super::*;

#[test]
fn card_reward_candidates_match_wire_contract() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {
                "skip_available": true,
                "cards": [
                    {"id": "Uppercut", "name": "Uppercut"},
                    {"id": "Anger", "name": "Anger"}
                ]
            }
        }
    });

    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );

    let actual: Vec<_> = candidates
        .iter()
        .map(|candidate| {
            (
                candidate.kind.as_str(),
                candidate.action_id.as_str(),
                candidate.label.as_str(),
                candidate.target_required,
            )
        })
        .collect();

    assert_eq!(
        actual,
        vec![
            ("choose", "card_reward:0", "Uppercut", None),
            ("choose", "card_reward:1", "Anger", None),
            ("skip", "card_reward:skip", "Skip", None),
        ]
    );
}

#[test]
fn combat_reward_candidates_match_wire_contract() {
    let raw = json!({
        "available_commands": ["choose", "proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "COMBAT_REWARD",
            "choice_list": ["gold", "relic", "emerald_key", "potion", "card"]
        }
    });

    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );

    let actual: Vec<_> = candidates
        .iter()
        .map(|candidate| {
            (
                candidate.kind.as_str(),
                candidate.action_id.as_str(),
                candidate.label.as_str(),
                candidate.target_required,
            )
        })
        .collect();

    assert_eq!(
        actual,
        vec![
            ("choose", "combat_reward:gold:0", "Collect gold", None),
            ("choose", "combat_reward:relic:1", "Collect relic", None),
            (
                "choose",
                "combat_reward:emerald_key:2",
                "Collect emerald_key",
                None,
            ),
            ("choose", "combat_reward:potion:3", "Collect potion", None),
            ("choose", "combat_reward:card:4", "Collect card", None),
        ]
    );
}
