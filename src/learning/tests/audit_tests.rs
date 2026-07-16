use super::support::decision_case;
use crate::learning::action::SemanticAction;
use crate::learning::audit::{AuditRole, select_audit_cases};
use crate::learning::telemetry::RecordedRankedAction;

#[test]
fn audit_prioritizes_adverse_outcomes_and_excludes_other_runs() {
    let safe = decision_case("run-a", 1);
    let mut costly = decision_case("run-a", 2);
    costly.outcome.combat_hp_lost = Some(24);
    costly.outcome.turn_hp_lost = Some(12);
    let mut death = decision_case("run-a", 3);
    death.outcome.combat_won = Some(false);
    let other = decision_case("run-b", 4);

    let cases = [safe, costly.clone(), death.clone(), other];
    let selected = select_audit_cases(&cases, "run-a", 2);

    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].case.case_id, death.case_id);
    assert_eq!(selected[1].case.case_id, costly.case_id);
}

#[test]
fn audit_tie_breaks_by_case_id_for_reproducible_prompts() {
    let mut left = decision_case("run-a", 1);
    let mut right = decision_case("run-a", 2);
    left.case_id = "a".into();
    right.case_id = "b".into();
    let cases = [right, left];
    let selected = select_audit_cases(&cases, "run-a", 10);

    assert!(
        selected
            .windows(2)
            .all(|pair| pair[0].case.case_id < pair[1].case.case_id)
    );
}

#[test]
fn victory_audit_anchors_the_direct_terminal_success() {
    let mut earlier = decision_case("run-a", 1);
    earlier.case_id = "earlier".into();
    earlier.decision_id = "run-a:51:8:20".into();
    let mut terminal = decision_case("run-a", 2);
    terminal.case_id = "terminal".into();
    terminal.decision_id = "run-a:51:9:21".into();
    terminal.outcome.alive_monsters_after_action = Some(0);
    terminal.outcome.player_died_after_action = Some(false);

    let cases = [earlier, terminal];
    let selected = select_audit_cases(&cases, "run-a", 2);

    assert_eq!(selected[0].case.case_id, "terminal");
    assert_eq!(selected[0].role, AuditRole::DirectVictoryTransition);
}

#[test]
fn defeat_audit_prioritizes_the_terminal_action_and_harmful_disagreement() {
    let defend = SemanticAction::PlayCard {
        card_id: "Defend_R".into(),
        upgraded: false,
        target_monster_id: None,
    };
    let perfected_strike = SemanticAction::PlayCard {
        card_id: "Perfected Strike".into(),
        upgraded: false,
        target_monster_id: Some("TheGuardian".into()),
    };

    let mut incidental = decision_case("run-a", 1);
    incidental.case_id = "incidental-overblock".into();
    incidental.decision_id = "run-a:16:10:91".into();
    incidental.selected_action = defend.clone();
    incidental.outcome.combat_won = Some(false);
    incidental.outcome.run_victory = Some(false);
    incidental.outcome.turn_hp_lost = Some(0);
    incidental.ranked_suggestions = vec![
        RecordedRankedAction {
            semantic_action: SemanticAction::EndTurn,
            score: 0,
            tags: vec![],
        },
        RecordedRankedAction {
            semantic_action: defend.clone(),
            score: -2_010,
            tags: vec!["excessive".into()],
        },
    ];

    let mut missed_block = decision_case("run-a", 2);
    missed_block.case_id = "missed-block".into();
    missed_block.decision_id = "run-a:16:9:87".into();
    missed_block.outcome.combat_won = Some(false);
    missed_block.outcome.run_victory = Some(false);
    missed_block.outcome.action_hp_lost = Some(0);
    missed_block.outcome.turn_hp_lost = Some(12);
    missed_block.ranked_suggestions = vec![
        RecordedRankedAction {
            semantic_action: defend.clone(),
            score: 166,
            tags: vec!["block".into()],
        },
        RecordedRankedAction {
            semantic_action: SemanticAction::EndTurn,
            score: 0,
            tags: vec![],
        },
    ];

    let mut fatal = decision_case("run-a", 3);
    fatal.case_id = "fatal-perfect-strike".into();
    fatal.decision_id = "run-a:16:12:99".into();
    fatal.selected_action = perfected_strike.clone();
    fatal.outcome.combat_won = Some(false);
    fatal.outcome.run_victory = Some(false);
    fatal.outcome.turn_hp_lost = Some(2);
    fatal.outcome.action_hp_lost = Some(2);
    fatal.outcome.player_died_after_action = Some(true);
    fatal.ranked_suggestions = vec![
        RecordedRankedAction {
            semantic_action: defend,
            score: 2_740,
            tags: vec!["block".into()],
        },
        RecordedRankedAction {
            semantic_action: perfected_strike,
            score: -386_745,
            tags: vec!["lethal".into(), "retaliation".into()],
        },
    ];

    let cases = [incidental, fatal, missed_block];
    let selected = select_audit_cases(&cases, "run-a", 3);

    assert_eq!(selected[0].case.case_id, "fatal-perfect-strike");
    assert_eq!(selected[0].role, AuditRole::DirectDeathTransition);
    assert_eq!(selected[1].case.case_id, "missed-block");
    assert_eq!(selected[1].role, AuditRole::HarmfulRankerDisagreement);
    assert_eq!(selected[2].case.case_id, "incidental-overblock");
}
