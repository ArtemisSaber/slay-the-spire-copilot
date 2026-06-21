use super::*;
use serde_json::json;

fn make_state(screen_type: &str, monsters: Option<Vec<serde_json::Value>>) -> serde_json::Value {
    let mut state = json!({
        "in_game": true,
        "game_state": {
            "screen_type": screen_type,
            "action_phase": "WAITING_ON_USER",
            "floor": 1,
            "room_type": "MonsterRoom",
        }
    });
    if let Some(monster_list) = monsters {
        state["game_state"]["combat_state"] = json!({
            "monsters": monster_list,
            "turn": 1,
        });
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

fn event_state_with_choices(choices: Vec<&str>) -> serde_json::Value {
    json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "floor": 8,
            "room_type": "EventRoom",
            "screen_state": {
                "text": "A strange event appears.",
                "options": choices
            }
        }
    })
}

fn with_monsters(screen_type: &str) -> serde_json::Value {
    make_state(screen_type, Some(vec![active_monster()]))
}

fn without_monsters(screen_type: &str) -> serde_json::Value {
    make_state(screen_type, Some(vec![]))
}

fn menu_state() -> serde_json::Value {
    json!({"in_game": false})
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
fn game_over_screen_ends_run() {
    let state = make_state("GAME_OVER", None);

    assert!(is_game_over_state(&state));
    assert!(should_end_run(&state, true));
    assert_eq!(run_end_reason(&state, true), Some("game_over"));
}

#[test]
fn leaving_game_after_observed_state_ends_run() {
    let state = menu_state();

    assert!(should_end_run(&state, true));
    assert_eq!(run_end_reason(&state, true), Some("left_game"));
}

#[test]
fn menu_before_any_observed_state_does_not_end_run() {
    let state = menu_state();

    assert!(!should_end_run(&state, false));
    assert_eq!(run_end_reason(&state, false), None);
}

#[test]
fn generate_screens_produce_advice() {
    for &screen in SCREEN_CONFIG.generate {
        if screen == "EVENT" {
            continue;
        }
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
fn boss_reward_generates_advice() {
    assert!(should_generate_advice(
        "BOSS_REWARD",
        &no_combat_state("BOSS_REWARD")
    ));
}

#[test]
fn event_with_multiple_choices_generates_advice() {
    assert!(should_generate_advice(
        "EVENT",
        &event_state_with_choices(vec!["Take", "Leave"])
    ));
}

#[test]
fn event_with_single_choice_does_not_generate_advice() {
    assert!(!should_generate_advice(
        "EVENT",
        &event_state_with_choices(vec!["Continue"])
    ));
}

#[test]
fn combat_entry_generates_advice_on_player_turn_start() {
    let mut gate = CombatTurnGate::new();
    let state = with_monsters("NONE");

    assert!(gate.is_player_turn_start(&state));
}

#[test]
fn combat_same_turn_does_not_generate() {
    let mut gate = CombatTurnGate::new();
    let state = with_monsters("NONE");

    assert!(gate.is_player_turn_start(&state));
    assert!(!gate.is_player_turn_start(&state));
}

#[test]
fn combat_entry_requires_waiting_on_user() {
    let mut gate = CombatTurnGate::new();

    assert!(!gate.is_player_turn_start(&without_monsters("NONE")));
    assert!(!gate.is_player_turn_start(&make_state("NONE", Some(vec![gone_monster()]))));

    let mut non_waiting = with_monsters("NONE");
    non_waiting["game_state"]["action_phase"] = json!("EXECUTING_ACTIONS");
    assert!(!gate.is_player_turn_start(&non_waiting));
}

#[test]
fn combat_new_turn_generates_advice() {
    let mut gate = CombatTurnGate::new();
    let state = with_monsters("NONE");
    assert!(gate.is_player_turn_start(&state));

    let mut state2 = state.clone();
    state2["game_state"]["combat_state"]["turn"] = json!(2);
    assert!(gate.is_player_turn_start(&state2));
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

#[tokio::test]
async fn finalize_run_writes_postmortem_report() {
    let dir = tempfile::tempdir().unwrap();
    let mut journal = crate::journal::Journal::new_at(dir.path(), "run-1");
    let config = crate::config::Config::from_env();
    let raw = serde_json::from_str::<serde_json::Value>(include_str!(
        "../../tests/fixtures/combat-state.json"
    ))
    .unwrap();
    let state = crate::state::NormalizedState::from_raw(&raw, &crate::test_utils::test_locale());
    let provider = crate::llm::LlmProvider::Mock;
    let mut finalized = false;

    journal.log_run_started_with_config(&config);
    journal.log_state_change(&state.stable_hash(), &state);
    finalize_run_once(
        &journal,
        &provider,
        "game_over",
        &mut finalized,
        &crate::test_utils::test_locale(),
    )
    .await;

    assert!(finalized);
    let report = std::fs::read_to_string(dir.path().join("run-1").join("postmortem.md")).unwrap();
    assert!(report.contains("# 本局复盘"));
    let events = std::fs::read_to_string(journal.path()).unwrap();
    assert!(events.contains("\"event\":\"run_ended\""));
    assert!(events.contains("\"reason\":\"game_over\""));
}

#[tokio::test]
async fn finalize_run_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let journal = crate::journal::Journal::new_at(dir.path(), "run-1");
    let config = crate::config::Config::from_env();
    let provider = crate::llm::LlmProvider::Mock;
    let mut finalized = false;

    journal.log_run_started_with_config(&config);
    finalize_run_once(
        &journal,
        &provider,
        "game_over",
        &mut finalized,
        &crate::test_utils::test_locale(),
    )
    .await;
    finalize_run_once(
        &journal,
        &provider,
        "stdin_closed",
        &mut finalized,
        &crate::test_utils::test_locale(),
    )
    .await;

    let events = std::fs::read_to_string(journal.path()).unwrap();
    assert_eq!(events.matches("\"event\":\"run_ended\"").count(), 1);
    assert!(events.contains("\"reason\":\"game_over\""));
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
