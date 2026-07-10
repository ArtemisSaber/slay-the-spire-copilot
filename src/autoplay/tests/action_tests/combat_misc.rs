use super::*;

#[test]
fn combat_rejects_unaffordable_requested_card() {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 1, "block": 0, "powers": []},
                "hand": [
                    {"id": "Bash", "name": "Bash", "cost": 2, "type": "ATTACK", "uuid": "bash-1", "has_target": true}
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
            &targeted_request("play", "combat:play:bash-1", 0)
        ),
        None
    );
}
