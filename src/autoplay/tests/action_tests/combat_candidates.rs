use super::*;

#[test]
fn combat_request_plays_requested_card_and_target() {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK", "uuid": "strike-1", "has_target": true},
                    {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "defend-1", "has_target": false}
                ],
                "monsters": [
                    {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                ]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_combat = true;

    assert_eq!(
        resolve(
            &raw,
            &control,
            &targeted_request("play", "combat:play:strike-1", 0)
        ),
        Some(AutoPlayAction::Play {
            hand_index: 0,
            target_index: Some(0),
        })
    );
}

#[test]
fn combat_candidates_require_targets_for_attacks() {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 1, "block": 0, "powers": []},
                "hand": [
                    {"id": "Bash", "name": "Bash", "cost": 2, "type": "ATTACK", "uuid": "bash-1", "has_target": true},
                    {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "defend-1", "has_target": false}
                ],
                "monsters": [
                    {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                ]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_combat = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );

    assert!(candidates.iter().any(|candidate| {
        candidate.action_id == "combat:play:bash-1" && candidate.target_required == Some(true)
    }));
    assert!(candidates.iter().any(|candidate| {
        candidate.action_id == "combat:play:defend-1" && candidate.target_required.is_none()
    }));
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.action_id == "combat:end")
    );
}

#[test]
fn combat_skill_with_has_target_needs_target() {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [
                    {"id": "Neutralize", "name": "Neutralize", "cost": 0, "type": "SKILL", "uuid": "neut-1", "has_target": true, "is_playable": true}
                ],
                "monsters": [
                    {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                ]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_combat = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    let neut = candidates
        .iter()
        .find(|c| c.action_id == "combat:play:neut-1")
        .unwrap();
    assert_eq!(
        neut.target_required,
        Some(true),
        "has_target skill should require target"
    );
}

#[test]
fn combat_skill_without_has_target_does_not_need_target() {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [
                    {"id": "Defend", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "def-1", "has_target": false, "is_playable": true}
                ],
                "monsters": [
                    {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                ]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_combat = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    let defend = candidates
        .iter()
        .find(|c| c.action_id == "combat:play:def-1")
        .unwrap();
    assert_eq!(
        defend.target_required, None,
        "defend has no target, should not require one"
    );
}

#[test]
fn combat_candidates_skip_unplayable_card() {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [
                    {"id": "Strike", "name": "Strike", "cost": 1, "type": "ATTACK", "uuid": "s-1", "has_target": true, "is_playable": false}
                ],
                "monsters": [
                    {"name": "Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                ]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_combat = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].action_id, "combat:end");
}

#[test]
fn combat_candidates_skip_card_no_uuid() {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [
                    {"id": "Strike", "name": "Strike", "cost": 1, "type": "ATTACK", "has_target": true, "is_playable": true}
                ],
                "monsters": [
                    {"name": "Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                ]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_combat = true;
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.iter().all(|c| c.kind != "play"));
}

#[test]
fn candidate_has_target_required_none() {
    let c = candidate("choose", "id:0".into(), "label".into());
    assert_eq!(c.target_required, None);
}

#[test]
fn targeted_candidate_has_target_required_true() {
    let c = targeted_candidate("play", "id:1".into(), "label".into());
    assert_eq!(c.target_required, Some(true));
}
