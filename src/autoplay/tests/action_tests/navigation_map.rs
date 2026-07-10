use super::*;

#[test]
fn map_request_maps_requested_index() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "MAP",
            "choice_list": ["M", "?"]
        }
    });
    let control = AutoPlayControl::default_enabled();

    assert_eq!(
        resolve(&raw, &control, &request("choose", "map:choice:1")),
        Some(AutoPlayAction::Choose(1))
    );
}

#[test]
fn map_candidates_empty_without_choose() {
    let raw = json!({
        "available_commands": ["proceed"],
        "ready_for_command": true,
        "game_state": {"screen_type": "MAP", "choice_list": ["M", "?"]}
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
fn resolve_map_rejects_out_of_bounds() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "MAP", "choice_list": ["M"]}
    });
    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "map:choice:5")
        ),
        None
    );
}

#[test]
fn resolve_map_rejects_wrong_kind() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {"screen_type": "MAP", "choice_list": ["M"]}
    });
    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("skip", "map:choice:0")
        ),
        None
    );
}
