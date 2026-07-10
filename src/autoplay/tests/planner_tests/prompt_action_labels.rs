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
