use super::*;
use serde_json::json;

fn make_state(screen_type: &str, monsters: Option<Vec<serde_json::Value>>) -> serde_json::Value {
    let mut state = json!({
        "in_game": true,
        "game_state": {
            "screen_type": screen_type,
            "floor": 1,
            "room_type": "MonsterRoom",
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
fn rest_screen_generates_advice() {
    assert!(should_generate_advice("REST", &no_combat_state("REST")));
}

#[test]
fn card_reward_still_generates_advice() {
    assert!(should_generate_advice(
        "CARD_REWARD",
        &no_combat_state("CARD_REWARD")
    ));
}

#[test]
fn combat_entry_generates_advice_on_first_combat_state() {
    let mut gate = AdviceGate::new();
    let state = with_monsters("NONE");

    assert!(gate.should_generate("NONE", &state));
}

#[test]
fn combat_followup_state_does_not_generate_advice() {
    let mut gate = AdviceGate::new();
    let state = with_monsters("NONE");

    assert!(gate.should_generate("NONE", &state));
    assert!(!gate.should_generate("NONE", &state));
}

#[test]
fn combat_entry_requires_active_monsters() {
    let mut gate = AdviceGate::new();

    assert!(!gate.should_generate("NONE", &without_monsters("NONE")));
    assert!(!gate.should_generate("NONE", &make_state("NONE", Some(vec![gone_monster()]))));
}

#[test]
fn startup_check_enabled_by_default() {
    let opts = runtime_options_from([], None);
    assert!(!opts.skip_startup_check);
    assert!(!opts.force_mock_provider);
}

#[test]
fn startup_check_skipped_by_no_startup_check_flag() {
    let opts = runtime_options_from(["--no-startup-check"], None);
    assert!(opts.skip_startup_check);
    assert!(!opts.force_mock_provider);
}

#[test]
fn startup_check_skipped_by_env_var() {
    let opts = runtime_options_from([], Some("1"));
    assert!(opts.skip_startup_check);
}

#[test]
fn stdin_test_mode_uses_mock_provider_by_default() {
    let opts = runtime_options_from(["--stdin-test"], None);
    assert!(opts.skip_startup_check);
    assert!(opts.force_mock_provider);
}

#[test]
fn postmortem_mode_uses_ai_by_default() {
    let opts = runtime_options_from(["postmortem", "runs/test/events.jsonl"], None);
    assert_eq!(
        opts.postmortem_path.as_deref(),
        Some("runs/test/events.jsonl")
    );
    assert!(!opts.postmortem_plain);
}

#[test]
fn postmortem_plain_flag_disables_ai_rewrite() {
    let opts = runtime_options_from(["postmortem", "--plain", "runs/test/events.jsonl"], None);
    assert_eq!(
        opts.postmortem_path.as_deref(),
        Some("runs/test/events.jsonl")
    );
    assert!(opts.postmortem_plain);
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
