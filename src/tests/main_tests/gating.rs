use super::*;

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
