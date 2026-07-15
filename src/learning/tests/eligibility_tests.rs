use crate::learning::eligibility::{
    IneligibilityReason, RunFacts, RunKind, RunObjective, RunOutcome, evaluate,
};

fn legitimate_facts(outcome: RunOutcome) -> RunFacts {
    RunFacts {
        ascension_level: Some(20),
        terminal_outcome: Some(outcome),
        character: Some("IRONCLAD".into()),
        seed: Some(42),
        objective: Some(RunObjective::Act3Victory),
        app_version: Some("0.2.0".into()),
        model_profile_sha256: Some("model".into()),
        prompt_schema_version: Some(1),
        locale: Some("en".into()),
        telemetry_consistent: true,
        ..RunFacts::default()
    }
}

#[test]
fn completed_wins_and_defeats_are_eligible_without_operator_setup() {
    for outcome in [RunOutcome::Victory, RunOutcome::Defeat] {
        let result = evaluate(&legitimate_facts(outcome));
        assert_eq!(result.run_kind, RunKind::Legitimate);
        assert!(result.knowledge_eligible);
        assert!(result.reasons.is_empty());
    }
}

#[test]
fn debug_ascension_is_valid_flow_but_never_knowledge() {
    let mut facts = legitimate_facts(RunOutcome::Victory);
    facts.ascension_level = Some(-15);

    let result = evaluate(&facts);

    assert_eq!(result.run_kind, RunKind::DebugFlow);
    assert!(!result.knowledge_eligible);
    assert!(
        result
            .reasons
            .contains(&IneligibilityReason::DebugAscension)
    );
}

#[test]
fn missing_terminal_outcome_is_incomplete() {
    let mut facts = legitimate_facts(RunOutcome::Victory);
    facts.terminal_outcome = None;

    let result = evaluate(&facts);

    assert_eq!(result.run_kind, RunKind::Incomplete);
    assert!(!result.knowledge_eligible);
    assert!(
        result
            .reasons
            .contains(&IneligibilityReason::MissingTerminalOutcome)
    );
}

#[test]
fn synthetic_restore_and_undo_runs_fail_closed() {
    let mut facts = legitimate_facts(RunOutcome::Victory);
    facts.synthetic_input = true;
    facts.used_restore = true;
    facts.used_undo = true;

    let result = evaluate(&facts);

    assert_eq!(result.run_kind, RunKind::Synthetic);
    assert!(!result.knowledge_eligible);
    assert!(
        result
            .reasons
            .contains(&IneligibilityReason::SyntheticInput)
    );
    assert!(result.reasons.contains(&IneligibilityReason::StateRestore));
    assert!(result.reasons.contains(&IneligibilityReason::UndoUsed));
}

#[test]
fn missing_required_provenance_is_unknown_and_lists_each_reason() {
    let facts = RunFacts {
        ascension_level: Some(0),
        terminal_outcome: Some(RunOutcome::Victory),
        telemetry_consistent: true,
        ..RunFacts::default()
    };

    let result = evaluate(&facts);

    assert_eq!(result.run_kind, RunKind::Unknown);
    assert!(!result.knowledge_eligible);
    assert!(
        result
            .reasons
            .contains(&IneligibilityReason::MissingCharacter)
    );
    assert!(result.reasons.contains(&IneligibilityReason::MissingSeed));
    assert!(
        result
            .reasons
            .contains(&IneligibilityReason::MissingObjective)
    );
}
