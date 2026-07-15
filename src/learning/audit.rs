use crate::learning::case::DecisionCase;

pub fn select_audit_cases<'a>(
    cases: &'a [DecisionCase],
    run_id: &str,
    maximum: usize,
) -> Vec<&'a DecisionCase> {
    let mut selected: Vec<_> = cases.iter().filter(|case| case.run_id == run_id).collect();
    selected.sort_by(|left, right| {
        priority(right)
            .cmp(&priority(left))
            .then_with(|| left.case_id.cmp(&right.case_id))
    });
    selected.truncate(maximum);
    selected
}

fn priority(case: &DecisionCase) -> u8 {
    if case.outcome.combat_won == Some(false) {
        return 5;
    }
    if case.outcome.combat_hp_lost.is_some_and(|loss| loss >= 15) {
        return 4;
    }
    if !case.retrieved_memory_ids.is_empty() {
        return 3;
    }
    if case
        .ranked_suggestions
        .first()
        .is_some_and(|ranked| ranked.semantic_action != case.selected_action)
    {
        return 2;
    }
    if case.outcome.turn_hp_lost.is_some_and(|loss| loss > 0) {
        return 1;
    }
    0
}
