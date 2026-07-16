use super::*;

#[test]
fn neow_event_produces_candidates() {
    let raw = json!({
        "available_commands": ["choose", "key", "click", "wait", "state"],
        "ready_for_command": true,
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["对话"],
            "screen_state": {
                "event_id": "Neow Event",
                "event_name": "涅奥",
                "options": [{"choice_index": 0, "disabled": false, "text": "[对话]", "label": "对话"}]
            },
            "class": "WATCHER",
            "current_hp": 72
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = command_state(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );
    assert!(!candidates.is_empty(), "expected candidates for Neow event");
    assert_eq!(candidates[0].action_id, "event:0");
}

#[test]
fn prompt_candidates_use_short_refs_without_execution_ids() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "EVENT", "choice_list": ["Take gold", "Leave"]}
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = command_state(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );
    let locale = Locale::load("en");
    let expected_scenario = crate::prompt::build_prompt(&state, &locale, false);

    let prompt = build_planner_prompt(
        &AutoPlaySession::default(),
        &command_state,
        &state,
        &locale,
        false,
        &candidates,
        &[],
    )
    .unwrap();
    let payload: Value = serde_json::from_str(&prompt).unwrap();
    let actions = payload["available_actions"].as_array().unwrap();

    assert_eq!(actions[0]["ref"], "A0");
    assert_eq!(actions[1]["ref"], "A1");
    assert_eq!(actions[0]["kind"], "choose");
    assert!(
        actions
            .iter()
            .all(|action| action.get("action_id").is_none())
    );
    assert_eq!(payload["schema"]["schema_version"], 2);
    assert!(payload["task"].as_str().unwrap().contains("bare action"));
    assert_eq!(payload["localized_status_context"], expected_scenario);
}

#[test]
fn combat_prompt_omits_card_uuid_from_the_complete_payload() {
    let secret_uuid = "run-local-secret-card-instance-42";
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 1, "block": 0, "powers": []},
                "hand": [{
                    "id": "Strike_R",
                    "name": "Strike",
                    "cost": 1,
                    "type": "ATTACK",
                    "uuid": secret_uuid,
                    "description": "Deal 6 damage.",
                    "has_target": true,
                    "is_playable": true
                }],
                "monsters": [{
                    "id": "JawWorm",
                    "name": "Jaw Worm",
                    "current_hp": 40,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "ATTACK",
                    "move_adjusted_damage": 11,
                    "move_hits": 1,
                    "is_gone": false
                }]
            }
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = command_state(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );

    assert!(
        candidates[0].action_id.contains(secret_uuid),
        "execution must retain the private card identity"
    );

    let prompt = build_planner_prompt(
        &AutoPlaySession::default(),
        &command_state,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        &[],
    )
    .unwrap();
    let payload: Value = serde_json::from_str(&prompt).unwrap();
    let card = &payload["state"]["hand"][0];

    assert!(!prompt.contains(secret_uuid));
    assert!(!prompt.to_ascii_lowercase().contains("uuid"));
    assert!(card.get("uuid").is_none());
    assert_eq!(card["index"], 0);
    assert_eq!(card["id"], "Strike_R");
    assert_eq!(card["name"], "Strike");
    assert_eq!(card["cost"], 1);
    assert_eq!(payload["available_actions"][0]["ref"], "A0");
}

#[test]
fn map_actions_include_position_labels() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "MAP",
            "choice_list": ["x=3", "x=4"],
            "map": {"current_x": 1, "current_y": 0, "first_node_chosen": true},
            "nodes": [
                {"x": 3, "y": 1, "symbol": "E", "children": []},
                {"x": 4, "y": 1, "symbol": "$", "children": []}
            ],
            "class": "WATCHER",
            "current_hp": 50,
            "max_hp": 72
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = command_state(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );

    let prompt = build_planner_prompt(
        &AutoPlaySession::default(),
        &command_state,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        &[],
    )
    .unwrap();

    assert!(
        prompt.contains("\"label\": \"(left) x=3\""),
        "MAP actions should include position label. Prompt:\n{prompt}"
    );
    assert!(
        prompt.contains("\"label\": \"(right) x=4\""),
        "MAP actions should include position label. Prompt:\n{prompt}"
    );
}

#[test]
fn non_map_actions_do_not_get_position_labels() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Option A", "Option B"],
            "class": "WATCHER",
            "current_hp": 72,
            "max_hp": 72
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command_state = command_state(&raw);
    let state = state(raw);
    let candidates = available_action_candidates(
        &control,
        &AutoPlaySession::default(),
        &command_state,
        &state,
    );

    let prompt = build_planner_prompt(
        &AutoPlaySession::default(),
        &command_state,
        &state,
        &Locale::load("en"),
        false,
        &candidates,
        &[],
    )
    .unwrap();

    assert!(
        prompt.contains("\"label\": \"Option A\""),
        "event actions should keep original labels. Prompt:\n{prompt}"
    );
    assert!(
        prompt.contains("\"label\": \"Option B\""),
        "event actions should keep original labels. Prompt:\n{prompt}"
    );
}
