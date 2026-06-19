use super::*;
use serde_json::json;

fn make_state(screen_type: &str, monsters: Option<Vec<serde_json::Value>>) -> serde_json::Value {
    let mut state = json!({
        "in_game": true,
        "game_state": {
            "screen_type": screen_type,
        }
    });
    if let Some(monster_list) = monsters {
        state["game_state"]["combat_state"] = json!({"monsters": monster_list});
    }
    state
}

fn active_monster() -> serde_json::Value {
    json!({"id": "JawWorm", "name": "Jaw Worm", "is_gone": false})
}

fn gone_monster() -> serde_json::Value {
    json!({"id": "JawWorm", "name": "Jaw Worm", "is_gone": true})
}

fn no_combat_state(screen_type: &str) -> serde_json::Value {
    make_state(screen_type, None)
}

fn with_monsters(screen_type: &str) -> serde_json::Value {
    make_state(screen_type, Some(vec![active_monster()]))
}

fn without_monsters(screen_type: &str) -> serde_json::Value {
    make_state(screen_type, Some(vec![]))
}

#[test]
fn has_monsters_detects_active() {
    assert!(has_monsters(&with_monsters("NONE")));
}

#[test]
fn has_monsters_ignores_gone() {
    let state = make_state("NONE", Some(vec![gone_monster()]));
    assert!(!has_monsters(&state));
}

#[test]
fn has_monsters_false_when_no_combat_state() {
    assert!(!has_monsters(&no_combat_state("NONE")));
}

#[test]
fn generate_screens_produce_advice() {
    for &screen in SCREEN_CONFIG.generate {
        assert!(
            should_generate_advice(screen, &no_combat_state(screen)),
            "{screen} should generate advice"
        );
    }
}

#[test]
fn generate_on_combat_screens_need_monsters() {
    for &screen in SCREEN_CONFIG.generate_on_combat {
        assert!(
            should_generate_advice(screen, &with_monsters(screen)),
            "{screen} should generate advice with monsters"
        );
        assert!(
            !should_generate_advice(screen, &without_monsters(screen)),
            "{screen} should not generate advice without monsters"
        );
    }
}

#[test]
fn screens_not_in_config_dont_generate() {
    let unconfigured = &[
        "COMBAT_REWARD",
        "REST",
        "SHOP",
        "BOSS_REWARD",
        "EVENT",
        "MAP",
        "GAME_OVER",
        "HAND_SELECT",
        "GRID",
        "UNKNOWN",
        "",
    ];
    for &screen in unconfigured {
        assert!(
            !should_generate_advice(screen, &no_combat_state(screen)),
            "{screen} should not generate advice"
        );
    }
}
