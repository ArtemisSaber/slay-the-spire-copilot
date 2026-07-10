use super::*;

#[test]
fn boss_reward_request_chooses_requested_relic() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "BOSS_REWARD",
            "screen_state": {
                "relics": [
                    {"id": "Coffee Dripper", "name": "Coffee Dripper"},
                    {"id": "Astrolabe", "name": "Astrolabe"}
                ]
            }
        }
    });

    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "boss_relic:1")
        ),
        Some(AutoPlayAction::Choose(1))
    );
}

#[test]
fn boss_reward_candidates_empty_without_choose() {
    let raw = json!({
        "available_commands": ["proceed"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "BOSS_REWARD",
            "screen_state": {"relics": [{"id": "Runic Dome", "name": "Runic Dome"}]}
        }
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}

#[test]
fn resolve_boss_reward_rejects_wrong_kind() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "BOSS_REWARD",
            "screen_state": {"relics": [{"id": "Runic Dome", "name": "Runic Dome"}]}
        }
    });
    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("skip", "boss_relic:0")
        ),
        None
    );
}

#[test]
fn resolve_boss_reward_rejects_out_of_bounds() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "BOSS_REWARD",
            "screen_state": {"relics": [{"id": "Runic Dome", "name": "Runic Dome"}]}
        }
    });
    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "boss_relic:5")
        ),
        None
    );
}
