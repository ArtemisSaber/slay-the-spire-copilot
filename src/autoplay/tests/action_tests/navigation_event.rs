use super::*;

#[test]
fn event_request_maps_requested_index() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Fight", "Leave"]
        }
    });

    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "event:1")
        ),
        Some(AutoPlayAction::Choose(1))
    );
}

#[test]
fn candidates_include_all_current_event_choices() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Fight", "Leave"]
        }
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );

    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.action_id.as_str())
            .collect::<Vec<_>>(),
        vec!["event:0", "event:1"]
    );
}

#[test]
fn event_candidates_exclude_disabled_visual_options_and_keep_wire_indices() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Heal", "Leave"],
            "screen_state": {
                "event_id": "The Cleric",
                "options": [
                    {
                        "choice_index": 0,
                        "disabled": false,
                        "text": "[Heal] 35 Gold: Heal 20 HP.",
                        "label": "Heal"
                    },
                    {
                        "disabled": true,
                        "text": "[Locked] Requires 50 Gold.",
                        "label": "Locked"
                    },
                    {
                        "choice_index": 1,
                        "disabled": false,
                        "text": "[Leave]",
                        "label": "Leave"
                    }
                ]
            }
        }
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw.clone()),
    );

    assert_eq!(
        candidates
            .iter()
            .map(|candidate| (candidate.action_id.as_str(), candidate.label.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("event:0", "[Heal] 35 Gold: Heal 20 HP."),
            ("event:1", "[Leave]"),
        ]
    );
    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "event:1")
        ),
        Some(AutoPlayAction::Choose(1))
    );
    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "event:2")
        ),
        None
    );
}

#[test]
fn event_candidates_fail_closed_when_normalized_choices_outnumber_wire_choices() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Heal", "Leave"]
        }
    });
    let mut normalized = state(raw.clone());
    normalized.event_choices = vec![
        "Heal 20 HP".into(),
        "Locked: requires 50 Gold".into(),
        "Leave".into(),
    ];
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &normalized,
    );

    assert_eq!(
        candidates
            .iter()
            .map(|candidate| (candidate.action_id.as_str(), candidate.label.as_str()))
            .collect::<Vec<_>>(),
        vec![("event:0", "Heal"), ("event:1", "Leave")]
    );
    assert_eq!(
        resolve_requested_action(
            &AutoPlayControl::default_enabled(),
            &AutoPlaySession::default(),
            &command_state(&raw),
            &normalized,
            &request("choose", "event:2"),
        ),
        None
    );
}

#[test]
fn event_candidates_empty_without_choose() {
    let raw = json!({
        "available_commands": ["proceed"],
        "ready_for_command": true,
        "game_state": {"screen_type": "EVENT", "choice_list": ["Fight"]}
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
fn resolve_event_rejects_out_of_bounds() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "EVENT", "choice_list": ["Fight"]}
    });
    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "event:5")
        ),
        None
    );
}

#[test]
fn resolve_event_rejects_wrong_kind() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "EVENT", "choice_list": ["Fight"]}
    });
    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("skip", "event:0")
        ),
        None
    );
}

#[test]
fn event_candidates_empty_choices_with_choose_command() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "EVENT", "choice_list": []}
    });
    let candidates = available_action_candidates(
        &AutoPlayControl::default_enabled(),
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
    );
    assert!(candidates.is_empty());
}
