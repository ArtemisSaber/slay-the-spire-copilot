use super::*;

#[test]
fn invalid_requested_action_is_rejected() {
    let raw = json!({
        "available_commands": ["choose"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "EVENT",
            "choice_list": ["Leave"]
        }
    });

    assert_eq!(
        resolve(
            &raw,
            &AutoPlayControl::default_enabled(),
            &request("choose", "event:9")
        ),
        None
    );
}

// ─── resolve: mismatched kind passes early filter but fails later ──

#[test]
fn resolve_rejects_when_action_id_and_kind_mismatch_candidates() {
    let raw = json!({
        "available_commands": ["end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [],
                "monsters": [
                    {"name": "Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                ]
            }
        }
    });
    let mut control = AutoPlayControl::default_enabled();
    control.allow_combat = true;
    let result = resolve_requested_action(
        &control,
        &AutoPlaySession::default(),
        &command_state(&raw),
        &state(raw),
        &request("play", "combat:end"),
    );
    assert_eq!(result, None);
}

// ─── parse_index ──────────────────────────────────────────────────

#[test]
fn parse_index_valid() {
    assert_eq!(parse_index("prefix:42", "prefix:"), Some(42));
}

#[test]
fn parse_index_missing_prefix() {
    assert_eq!(parse_index("wrong:42", "prefix:"), None);
}

#[test]
fn parse_index_non_numeric() {
    assert_eq!(parse_index("prefix:abc", "prefix:"), None);
}

#[test]
fn parse_index_empty() {
    assert_eq!(parse_index("prefix:", "prefix:"), None);
}
