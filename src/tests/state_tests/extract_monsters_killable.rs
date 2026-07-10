use super::*;

#[test]
fn monster_can_be_killed_with_enough_hand_attack() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Weak Enemy",
                    "id": "WeakEnemy",
                    "current_hp": 6,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "ATTACK"
                }],
                "hand": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(state.monsters[0].can_be_killed);
}
