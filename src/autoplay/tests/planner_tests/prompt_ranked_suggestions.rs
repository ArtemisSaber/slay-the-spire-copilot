use super::*;

#[test]
fn prompt_includes_ranked_suggestions_for_combat() {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 1, "block": 0, "powers": []},
                "hand": [{
                    "name": "Strike", "id": "Strike_R", "cost": 1, "type": "ATTACK",
                    "uuid": "s1", "has_target": true, "is_playable": true,
                    "exhausts": false, "ethereal": false, "upgrades": 0, "rarity": "BASIC"
                }],
                "monsters": [{
                    "name": "Jaw Worm", "id": "JawWorm", "current_hp": 1, "max_hp": 46,
                    "block": 0, "intent": "ATTACK", "move_hits": 1, "move_base_damage": 12,
                    "move_adjusted_damage": 12, "is_gone": false, "half_dead": false,
                    "powers": [], "move_id": 1
                }],
                "turn": 1
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
        prompt.contains("ranked_suggestions"),
        "combat prompt should include ranked_suggestions"
    );
    let payload: Value = serde_json::from_str(&prompt).unwrap();
    let available_refs: Vec<_> = payload["available_actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|action| action["ref"].clone())
        .collect();
    let ranked = payload["ranked_suggestions"].as_array().unwrap();
    assert!(!ranked.is_empty());
    assert!(
        ranked
            .iter()
            .all(|entry| available_refs.contains(&entry["ref"]))
    );
    assert!(ranked.iter().all(|entry| entry.get("action").is_none()));
    let strike = ranked
        .iter()
        .find(|entry| entry["label"] == "Play Strike")
        .unwrap();
    let end = ranked
        .iter()
        .find(|entry| entry["label"] == "End turn")
        .unwrap();
    assert_eq!(strike["ref"], "A0");
    assert_eq!(end["ref"], "A1");
}

#[test]
fn prompt_excludes_ranked_suggestions_for_non_combat() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "screen_state": {
                "skip_available": true,
                "cards": [{"id": "Anger", "name": "Anger"}]
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
        !prompt.contains("ranked_suggestions"),
        "non-combat prompt should not include ranked_suggestions"
    );
}
