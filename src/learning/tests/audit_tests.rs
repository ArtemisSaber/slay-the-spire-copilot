use super::support::decision_case;
use crate::learning::audit::select_audit_cases;

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
    assert_eq!(selected[0].case_id, death.case_id);
    assert_eq!(selected[1].case_id, costly.case_id);
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
            .all(|pair| pair[0].case_id < pair[1].case_id)
    );
}
